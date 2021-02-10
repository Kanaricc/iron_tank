use std::{thread, time::Duration};

use lapin::{BasicProperties, Connection, ConnectionProperties, Result, options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions}, types::FieldTable};
use log::info;
use tokio::{task::spawn_blocking, time::sleep};
use tokio_amqp::*;
use futures_util::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default().with_tokio()).await?; // Note the `with_tokio()` here
    let channel_result = conn.create_channel().await?;
    let channel_task = conn.create_channel().await?;

    let _task_queue = channel_task
        .queue_declare(
            "judge_tasks",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    let mut consumer = channel_result
        .basic_consume(
            "judge_results",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let t=tokio::spawn(async move {
        while let Some(delivery) = consumer.next().await {
            tokio::time::sleep(Duration::from_millis(300)).await;
            let delivery = delivery.expect("error in consumer");
            delivery
                .1
                .ack(BasicAckOptions::default())
                .await
                .expect("ack");

            channel_task
                .basic_publish(
                    "",
                    "judge_tasks",
                    BasicPublishOptions::default(),
                    "233".to_string().into_bytes(),
                    BasicProperties::default(),
                )
                .await.unwrap()
                .await.unwrap();
        }
    });

    t.await.unwrap();

    Ok(())
}
