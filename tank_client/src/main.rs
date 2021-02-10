use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions},
    types::FieldTable,
    Connection, ConnectionProperties, Result,
};
use tokio_amqp::*;
use futures_util::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let conn = Connection::connect(&addr, ConnectionProperties::default().with_tokio()).await?; // Note the `with_tokio()` here
    let channel = conn.create_channel().await?;

    let mut consumer = channel
        .basic_consume(
            "judge_tasks",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("error in consumer").1;
        delivery.ack(BasicAckOptions::default()).await.expect("ack");
    }
    Ok(())
}
