use std::collections::{VecDeque, HashMap};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify, mpsc};
use tauri::Emitter;
use crate::database::models::*;

#[derive(Debug, Clone)]
pub struct QueuedTask {
    pub task: DownloadTask,
    pub ytdlp_path: std::path::PathBuf,
    pub ffmpeg_path: Option<std::path::PathBuf>,
    pub settings: AppSettings,
    pub is_resume: bool,
}

pub struct DownloadQueue {
    queue: Mutex<VecDeque<QueuedTask>>,
    active_count: Mutex<u32>,
    max_concurrent: Mutex<u32>,
    notify: Notify,
    active_tasks_info: Mutex<HashMap<String, QueuedTask>>,
    paused_tasks: Mutex<HashMap<String, QueuedTask>>,
}

impl DownloadQueue {
    pub fn new(max_concurrent: u32) -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            active_count: Mutex::new(0),
            max_concurrent: Mutex::new(max_concurrent),
            notify: Notify::new(),
            active_tasks_info: Mutex::new(HashMap::new()),
            paused_tasks: Mutex::new(HashMap::new()),
        }
    }

    pub async fn set_max_concurrent(&self, max: u32) {
        *self.max_concurrent.lock().await = max;
        self.notify.notify_waiters();
    }

    pub async fn enqueue(&self, task: QueuedTask) {
        self.queue.lock().await.push_back(task);
        self.notify.notify_one();
    }

    pub async fn try_dequeue(&self) -> Option<QueuedTask> {
        let active = *self.active_count.lock().await;
        let max = *self.max_concurrent.lock().await;
        if active >= max { return None; }
        let task = self.queue.lock().await.pop_front();
        if let Some(ref t) = task {
            *self.active_count.lock().await += 1;
            self.active_tasks_info.lock().await.insert(t.task.id.clone(), t.clone());
        }
        task
    }

    pub async fn task_finished(&self, task_id: &str) {
        self.active_tasks_info.lock().await.remove(task_id);
        let mut count = self.active_count.lock().await;
        if *count > 0 { *count -= 1; }
        drop(count);
        self.notify.notify_one();
    }

    pub async fn move_to_paused(&self, task_id: &str) {
        if let Some(mut qt) = self.active_tasks_info.lock().await.remove(task_id) {
            qt.is_resume = true; // ★ 标记为恢复模式
            self.paused_tasks.lock().await.insert(task_id.to_string(), qt);
        }
        let mut count = self.active_count.lock().await;
        if *count > 0 { *count -= 1; }
        drop(count);
        self.notify.notify_one();
    }

    pub async fn take_paused(&self, task_id: &str) -> Option<QueuedTask> {
        self.paused_tasks.lock().await.remove(task_id)
    }

    pub async fn remove_task(&self, task_id: &str) -> bool {
        let mut q = self.queue.lock().await;
        let before = q.len();
        q.retain(|t| t.task.id != task_id);
        let removed = q.len() != before;
        drop(q);
        // 也从暂停列表移除
        self.paused_tasks.lock().await.remove(task_id);
        removed
    }

    pub async fn queue_size(&self) -> usize { self.queue.lock().await.len() }
    pub async fn active_count_val(&self) -> u32 { *self.active_count.lock().await }

    pub async fn wait_for_slot(&self) {
        loop {
            {
                let active = *self.active_count.lock().await;
                let max = *self.max_concurrent.lock().await;
                if active < max && !self.queue.lock().await.is_empty() { return; }
            }
            self.notify.notified().await;
        }
    }
}

pub fn start_queue_worker(
    queue: Arc<DownloadQueue>,
    process_manager: Arc<crate::ytdlp::process::ProcessManager>,
    db: Arc<crate::database::Database>,
    app_handle: tauri::AppHandle,
) {
    let queue_clone = queue.clone();
    tauri::async_runtime::spawn(async move {
        log::info!("Queue worker started");
        loop {
            queue_clone.wait_for_slot().await;

            if let Some(queued_task) = queue_clone.try_dequeue().await {
                let task = queued_task.task.clone();
                let is_resume = queued_task.is_resume;
                let pm = process_manager.clone();
                let db = db.clone();
                let app = app_handle.clone();
                let q = queue_clone.clone();

                log::info!("Queue: starting task {} (resume={}) - {}", task.id, is_resume, task.title);

                let _ = app.emit("download-progress", &DownloadProgress {
                    task_id: task.id.clone(), status: DownloadStatus::Downloading,
                    progress: 0.0, speed: None, eta: Some("启动中...".into()),
                    total_size: None, downloaded_size: None, output_path: None, error_message: None,
                });

                tauri::async_runtime::spawn(async move {
                    let (tx, mut rx) = mpsc::unbounded_channel::<DownloadProgress>();
                    let task_id = task.id.clone();
                    let task_title = task.title.clone();
                    let settings = queued_task.settings.clone();

                    let app_clone = app.clone();
                    let db_clone = db.clone();
                    let tid = task_id.clone();
                    tauri::async_runtime::spawn(async move {
                        while let Some(progress) = rx.recv().await {
                            let _ = app_clone.emit("download-progress", &progress);
                            match progress.status {
                                DownloadStatus::Completed => {
                                    let _ = db_clone.update_task_status(&tid,
                                        &serde_json::to_string(&DownloadStatus::Completed).unwrap(),
                                        progress.output_path.as_deref(), None,
                                        Some(&chrono::Local::now().to_rfc3339()));
                                }
                                DownloadStatus::Failed => {
                                    let _ = db_clone.update_task_status(&tid,
                                        &serde_json::to_string(&DownloadStatus::Failed).unwrap(),
                                        None, progress.error_message.as_deref(), None);
                                }
                                DownloadStatus::Paused => {
                                    let _ = db_clone.update_task_status(&tid,
                                        &serde_json::to_string(&DownloadStatus::Paused).unwrap(),
                                        None, None, None);
                                }
                                _ => {}
                            }
                        }
                    });

                    let result = pm.start_download(
                        &queued_task.ytdlp_path, &queued_task.ffmpeg_path,
                        &task, &queued_task.settings, is_resume, tx,
                    ).await;

                    match &result {
                        Ok(true) => {
                            if settings.notify_on_complete {
                                let _ = app.emit("notify", serde_json::json!({
                                    "type": "complete", "title": "下载完成", "body": task_title,
                                }));
                            }
                            q.task_finished(&task_id).await;
                        }
                        Ok(false) => {
                            // ★ 暂停：移入暂停列表
                            log::info!("Task {} paused → saved for resume", task_id);
                            q.move_to_paused(&task_id).await;
                        }
                        Err(_) => {
                            if settings.notify_on_error {
                                let _ = app.emit("notify", serde_json::json!({
                                    "type": "error", "title": "下载失败", "body": task_title,
                                }));
                            }
                            q.task_finished(&task_id).await;
                        }
                    }
                });
            }
        }
    });
}