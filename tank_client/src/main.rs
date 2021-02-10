use futures_util::stream::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, QueueDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties, Result,
};
use tokio_amqp::*;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default().with_tokio()).await?; // Note the `with_tokio()` here
    let channel_result = conn.create_channel().await?;
    let channel_task = conn.create_channel().await?;

    let _result_queue = channel_result.queue_declare(
        "judge_results",
        QueueDeclareOptions::default(),
        FieldTable::default(),
    );

    let mut consumer = channel_task
        .basic_consume(
            "judge_tasks",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumer");
        delivery
            .1
            .ack(BasicAckOptions::default())
            .await
            .expect("ack");

        channel_result
            .basic_publish(
                "",
                "judge_results",
                BasicPublishOptions::default(),
                "233".to_string().into_bytes(),
                BasicProperties::default(),
            )
            .await?
            .await?;
    }
    Ok(())
}
