use lapin::{BasicProperties, Connection, ConnectionProperties, Result, options::{BasicPublishOptions, QueueDeclareOptions}, types::FieldTable};
use log::info;
use tokio_amqp::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default().with_tokio()).await?; // Note the `with_tokio()` here
    let channel = conn.create_channel().await?;

    let task_queue = channel
        .queue_declare(
            "judge_tasks",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    info!("declare queue {:?}",task_queue);

    let t=channel.basic_publish(
        "",
        "judge_tasks",
        BasicPublishOptions::default(),
        "233".to_string().into_bytes(),
        BasicProperties::default(),
    ).await?.await?;

    println!("{:#?}",t);


    // Rest of your program

    Ok(())
}
