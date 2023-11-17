use rand::Rng;
use std::{
    error::Error,
    io,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
    vec,
};

#[derive(Debug)]
enum ItemType {
    Apple,
    Orange,
}

#[derive(Debug, Clone)]
struct Apple();

#[derive(Debug, Clone)]
struct Orange();

#[derive(Debug)]
struct FillContainerMessage<TItem> {
    container: Vec<TItem>,
}

#[derive(Debug)]
struct ContainerFilledMessage {
    container_size: usize,
    items_added: usize,
    item_type: ItemType,
}

fn main() -> Result<(), Box<dyn Error>> {
    let (apples_tx, apples_rx) = mpsc::channel::<FillContainerMessage<Apple>>();
    let (oranges_tx, oranges_rx) = mpsc::channel::<FillContainerMessage<Orange>>();
    let (ready_tx, ready_rx) = mpsc::channel::<ContainerFilledMessage>();

    let ready_tx_apples = ready_tx.clone();
    let ready_tx_oranges = ready_tx;

    let apples_thread = thread::spawn(move || collect_apples(apples_rx, ready_tx_apples));
    let oranges_thread = thread::spawn(move || collect_oranges(oranges_rx, ready_tx_oranges));
    let results_thread = thread::spawn(move || report_results(ready_rx));

    generate_work(apples_tx, oranges_tx)?;

    let apples_result = apples_thread.join();
    let oranges_result = oranges_thread.join();
    let results_result = results_thread.join();

    if let Err(apples_e) = apples_result {
        println!("Apples failed to be collected: {apples_e:?}");
    }

    if let Err(oranges_e) = oranges_result {
        println!("Oranges failed to be collected: {oranges_e:?}");
    }

    if let Err(results_e) = results_result {
        println!("Results failed to be reported: {results_e:?}");
    }

    Ok(())
}

fn generate_work(
    apples_tx: Sender<FillContainerMessage<Apple>>,
    oranges_tx: Sender<FillContainerMessage<Orange>>,
) -> Result<(), Box<dyn Error>> {
    println!("Press enter to give the app more work to do.");

    let mut rng = rand::thread_rng();

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // We do not care what the input is. We just generate more work every time enter is pressed.
        let item_type = if rng.gen_bool(0.5) {
            ItemType::Apple
        } else {
            ItemType::Orange
        };

        let container_size = rng.gen_range(1..10);

        match item_type {
            ItemType::Apple => {
                let container = vec![Apple {}; container_size];
                let send_result = apples_tx.send(FillContainerMessage { container });

                if send_result.is_err() {
                    // Work channel is closed, we cannot function in this mode.
                    return Ok(());
                }
            }
            ItemType::Orange => {
                let container = vec![Orange {}; container_size];
                let send_result = oranges_tx.send(FillContainerMessage { container });

                if send_result.is_err() {
                    // Work channel is closed, we cannot function in this mode.
                    return Ok(());
                }
            }
        }
    }
}

fn collect_apples(
    rx: Receiver<FillContainerMessage<Apple>>,
    ready_tx: Sender<ContainerFilledMessage>,
) {
    let mut rng = rand::thread_rng();

    for mut work_order in rx {
        thread::sleep(Duration::from_secs(1));

        let apples_collected = rng.gen_range(1..=work_order.container.len());

        for i in 0..apples_collected {
            work_order.container[i] = Apple {};
        }

        let send_result = ready_tx.send(ContainerFilledMessage {
            container_size: work_order.container.len(),
            items_added: apples_collected,
            item_type: ItemType::Apple,
        });

        if send_result.is_err() {
            // Result channel is closed, we cannot function in this mode.
            return;
        }
    }
}

fn collect_oranges(
    rx: Receiver<FillContainerMessage<Orange>>,
    ready_tx: Sender<ContainerFilledMessage>,
) {
    let mut rng = rand::thread_rng();

    for mut work_order in rx {
        thread::sleep(Duration::from_secs(2));

        let oranges_collected = rng.gen_range(1..=work_order.container.len());

        for i in 0..oranges_collected {
            work_order.container[i] = Orange {};
        }

        let send_result = ready_tx.send(ContainerFilledMessage {
            container_size: work_order.container.len(),
            items_added: oranges_collected,
            item_type: ItemType::Orange,
        });

        if send_result.is_err() {
            // Result channel is closed, we cannot function in this mode.
            return;
        }
    }
}

fn report_results(rx: Receiver<ContainerFilledMessage>) {
    for message in rx {
        println!(
            "Collected {}x {:?} into a container of size {}",
            message.items_added, message.item_type, message.container_size
        );
    }
}
