use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::task::JoinHandle;
use futures_util::future::join_all;

type AsyncFileHandle = Arc<Mutex<File>>;
type FileJoinHandle = JoinHandle<Result<bool, String>>;

struct AsyncWriteFuture {
    pub handle: AsyncFileHandle,
    pub entry: String
}

impl Future for AsyncWriteFuture {
    type Output = Result<bool, String>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut guard = match self.handle.try_lock() {
            Ok(guard) => guard,
            Err(error) => {
                println!("error for {} : {}", self.entry, error);
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        };
        
        let lined_entry = format!("{}\n", self.entry);
        match guard.write(lined_entry.as_bytes()) {
            Ok(_) => println!("written for: {}", self.entry),
            Err(_) => println!("error: {}", self.entry)
        };
        
        Poll::Ready(Ok(true))
    }
}


#[tokio::main]
async fn main() {
    
    let login_handle = get_handle(&"login.txt");
    let logout_handle = get_handle(&"logout.txt");
    
    let names = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine"];
    
    let mut handles = Vec::new();
    
    for name in names {
        let file_handle = login_handle.clone();
        let file_handle2 = logout_handle.clone();
        
        let handle = write_log(file_handle.clone(), name.to_string());
        let handle2 = write_log(file_handle2.clone(), name.to_string());
        
        handles.push(handle);
        handles.push(handle2);
    }
    
    let _ = join_all(handles).await;
}

fn get_handle(file_path: &dyn ToString) -> AsyncFileHandle {
    match OpenOptions::new().append(true).open(file_path.to_string()) {
        Ok(file) => {
            Arc::new(Mutex::new(file))
        }
        Err(_) => {
            Arc::new(Mutex::new(File::create(file_path.to_string()).unwrap()))
        }
    }
}

fn write_log(async_file_handle: AsyncFileHandle, line: String) -> FileJoinHandle {
    let future = AsyncWriteFuture {
        handle: async_file_handle,
        entry: line,
    };

    tokio::task::spawn(async move {
        future.await
    })
}