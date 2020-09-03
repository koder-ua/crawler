use crate::{utils, spmc};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::mpsc;
use std::thread;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

pub fn process_url(url: &str, root: &PathBuf) -> Vec<String> {
    match utils::load(&url) {
        Err(e) => {
            println!("Error while processing url {}: {}", utils::decode(&url), e.to_string());
            return vec![]
        },
        Ok(v) => {
            utils::save_page(root, &url, &v).unwrap_or_else(
                |err| println!("Failed to save {} - {}", utils::decode(&url), err.to_string()));
            return utils::all_links(&url, &v)
        }
    }
}

#[allow(dead_code)]
pub fn load_loop(initial_url: &str, save_dir: &PathBuf, max_level: usize) {
    let mut queue = VecDeque::new();
    let mut levels = HashMap::new();

    queue.push_front(initial_url.to_string());
    levels.insert(initial_url.to_string(), 0);
    while !queue.is_empty() {
        let url = queue.pop_front().unwrap();
        let level = *levels.get(&url).unwrap();
        println!("+{} {}", level, url);

        for link in &process_url(&url, save_dir) {
            if levels.contains_key(link) {
                continue
            }
            levels.insert(link.clone(), level + 1);
            if level + 1 != max_level {
                queue.push_front(link.clone());
            }
        }
    }
}

fn load_thread_func(tasks_q: spmc::Queue<String>, res_chan: Sender<(String, String)>) {
    loop {
        let url = tasks_q.recv();
        if url.is_empty() {
            break
        }
        match utils::load(&url) {
            Err(e) => {
                println!("Error while processing url {}: {}", utils::decode(&url), e.to_string());
            },
            Ok(v) => {
                println!("sending result for {}", utils::decode(&url));
                res_chan.send((url, v)).unwrap();
            }
        }
    }
}

fn start_threads(th_count: usize, queue: &spmc::Queue<String>, res_s: &Sender<(String, String)>) -> Vec<JoinHandle<()>> {
    let mut handles = vec![];

    for _ in 0..th_count {
        let qcopy = queue.clone();
        let curr_res_s: Sender<(String, String)> = res_s.clone();
        handles.push(thread::spawn(move || load_thread_func(qcopy, curr_res_s)));
    }

    return handles
}

pub fn load_loop_mt(initial_url: &str, save_dir: &PathBuf, max_level: usize, th_count: usize) {
    let (res_s, res_r) = mpsc::channel();
    let queue: spmc::Queue<String> = spmc::Queue::new();
    let mut in_progress: HashSet<String> = HashSet::new();
    let mut levels = HashMap::new();

    let handles = start_threads(th_count, &queue, &res_s);

    queue.send(initial_url.to_string());
    levels.insert(initial_url.to_string(), 0);
    in_progress.insert(initial_url.to_string());

    while !in_progress.is_empty() {
        let (url, data) = res_r.recv().unwrap();
        let level = *levels.get(&url).unwrap();
        println!("get result for {} {} bytes", utils::decode(&url), data.len());

        utils::save_page(save_dir, &url, &data)
            .unwrap_or_else(
                |err| println!("Failed to save {} - {}", utils::decode(&url), err.to_string()));

        in_progress.remove(&url);

        for link in utils::all_links(&url, &data) { //.into_iter().take(5).into_iter() {
            if !levels.contains_key(&link) {
                levels.insert(link.clone(), level + 1);
                if level + 1 != max_level {
                    println!("Scheduling {}", utils::decode(&link));
                    queue.send(link.clone());
                    in_progress.insert(link.clone());
                }
            }
        }
    }

    for _ in &handles {
        queue.send("".to_string());
    }

    for handler in handles {
        handler.join().unwrap();
    }
}

