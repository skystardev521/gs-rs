//use std::thread;

pub mod json;
pub mod proto;
pub mod test_tcp;
pub mod toml;
pub mod wan_tcp_rw;

#[macro_use]
use mini_utils::signal;

use hiredis_sys::RedisClient;
use mini_utils::logger::Logger;
use mini_utils::wtimer::TestIWTask;
use mini_utils::wtimer::WTimer;

use std::time::Duration;
use tokio::time::sleep;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

trait Hello {
    fn say_hi(&self);
}

struct TestHello {
    v: u16,
}

impl Default for TestHello {
    fn default() -> Self {
        TestHello { v: 123 }
    }
}

impl Hello for TestHello {
    fn say_hi(&self) {
        println!("say_hi:{}", self.v)
    }
}

impl TestHello {
    pub fn new() -> Self {
        TestHello { v: 0 }
    }
    pub fn test_mut(&self) -> &mut Self {
        unsafe { &mut *(self as *const Self as *mut Self) }
    }
}

struct TestTrait<T> {
    hello: Box<T>,
}

impl<T> TestTrait<T>
where
    T: Hello + Default,
{
    pub fn new() -> Self {
        TestTrait {
            hello: Box::new(T::default()),
        }
    }
}
use std::collections::VecDeque;
use std::mem::MaybeUninit;
struct tv {
    t: [VecDeque<usize>; 8],
}

pub fn test_redis_client() {
    //test_fmt!("test_fmt:{} {} {}", "a", "b", "c");
    match RedisClient::redis_connect_timeout(String::from("127.0.0.1"), 6379, 1000) {
        Ok(client) => {
            let mut n = 0;
            loop {
                n += 1;
                if n == 10 {
                    break;
                }
                let cmd = format!("HGETALL runoobkey");
                //let cmd = format!("hmset hkey k1 12345 k2 12345678");

                match client.redis_cmd_vec_str(cmd) {
                    Ok(vec_str) => println!("reply_ptr:{:?}", vec_str),
                    Err(err) => println!("redis_command err:{}", err),
                }
            }
        }
        Err(err) => println!("connect err:{}", err),
    }
}

#[test]
fn test_fn() {
    TestHello::new().test_mut();
    println!("xxx:");
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let (v1, v2, v3) = tokio::join!(
        async {
            sleep(Duration::from_millis(1500)).await;
            println!("Value 1 ready");
            "Value 1"
        },
        async {
            sleep(Duration::from_millis(2800)).await;
            println!("Value 2 ready");
            "Value 2"
        },
        async {
            sleep(Duration::from_millis(600)).await;
            println!("Value 3 ready");
            "Value 3"
        },
    );

    assert_eq!(v1, "Value 1");
    assert_eq!(v2, "Value 2");
    assert_eq!(v3, "Value 3");
    
    let mut listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}

/*
fn main() {

    
    match Logger::init(&String::from("info"), &String::from("logs/test_log.log")) {
        Ok(()) => (),
        Err(err) => println!("Logger::init error:{}", err),
    }

    test_tcp::test();

    test_redis_client();

    let mut wtimer = WTimer::new(1);

    for i in 0..9 {
        println!("name:{}", i);
        let task = Box::new(TestIWTask {
            id: 0,
            name: format!("name:{}", i),
        });
        wtimer.push_task(1, 10, task);
    }

    loop {
        //std::thread::sleep(std::time::Duration::from_millis(1));
        wtimer.scheduled(mini_utils::time::timestamp());
    }

    /*
    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());

    println!("Tevn is {} ", env!("PATH"));

    let th = Box::new(TestHello::default());

    let t: TestTrait<TestHello> = TestTrait::new();

    t.hello.say_hi();

    let xxx = vec![0u8; 10];
    */

    /*
        let mut thread_pool: Vec<thread::JoinHandle<()>> = vec![];

        for _ in 0..16 {
            thread_pool.push(thread::spawn(move || {
                for i in 0..99999999 {
                    info!("thread_id-->{}:{:?}", i, std::thread::current().id());
                    //std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }));
        }
    */

    //let mut hm: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();
}
*/