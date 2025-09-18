use once_cell::sync::Lazy;
use tokio::runtime::{self, Runtime};
use tokio::sync::mpsc::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::time::{sleep, Duration};
use tokio::sync::mpsc::error::TryRecvError;
use reqwest;
use http::StatusCode;
use std::ffi::CStr;
use std::os::raw::c_char;

static CHANNEL_MAP: Lazy<Arc<RwLock<HashMap<String, UnboundedSender<i32>>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(HashMap::new()))
});

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    println!("Building runtime");
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});

#[unsafe(no_mangle)]
pub extern "C" fn configure_probe(url_chars: *const c_char, interval: i32, id: i32, on_probe_result: extern "C" fn(i32, bool)) {
    let url_cstr = unsafe { CStr::from_ptr(url_chars) };
    let url = url_cstr.to_string_lossy().into_owned();
    let mut map = CHANNEL_MAP.write().unwrap();
    let channel = map.get(&url);
    
    match channel {
        Some(chan) => {
            let _ = chan.send(interval);
        }
        None => {
           let (chan, mut rcv) = unbounded_channel();
           map.insert(url.clone(), chan.clone());
            RUNTIME.spawn(async move {
                let mut probe_interval = interval;
                println!("Prober task spawned for id {}", id);
                loop {
                    let val = rcv.try_recv();
                    match val {
                        Err(TryRecvError::Disconnected) => {
                            println!("Exiting {}, no more workload for this prober.", id);
                            break;
                        }
                        Err(TryRecvError::Empty) => {
                            let mut succeeded = true;
                            sleep(Duration::from_secs(probe_interval.try_into().unwrap())).await;
                            let resp = reqwest::get(url.clone()).await;
                            match resp {
                                Ok(resp) => {
                                    if resp.status() == StatusCode::OK {
                                        succeeded = true;
                                        println!("probe succeeded for {}", url);
                                    }
                                    else {
                                        succeeded = false;
                                        println!("probe failed for {} with non-200 status", url);
                                    }
                                }
                                Err(_) => {
                                    succeeded = false;
                                    println!("probe failed for {}", url);
                                }
                            }
                            on_probe_result(id, succeeded);
                        }
                        Ok(val) => {
                            println!("Setting {} to probe at interval {}", url, val);
                            probe_interval = val;
                        }
                    }
                }      
            });
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn stop_probe(url_chars: *const c_char) {
    let url_cstr = unsafe { CStr::from_ptr(url_chars) };
    let url = url_cstr.to_string_lossy().into_owned();
    let mut map = CHANNEL_MAP.write().unwrap();
    let _ = map.remove(&url);
}