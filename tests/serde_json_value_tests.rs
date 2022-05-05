pub mod common;

pub use common::{features::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, entity::*, DatabaseConnection};
use serde_json::json;

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("serde_json_value_tests").await;
    create_tables(&ctx.db).await?;
    insert_serde_json_value(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn insert_serde_json_value(db: &DatabaseConnection) -> Result<(), DbErr> {
    use serde_json_value::*;

    let model = Model {
        id: 1,
        json: json!({
            "id": 1,
            "name": "apple",
            "price": 12.01,
            "notes": "hand picked, organic",
        }),
        json_value: KeyValue {
            id: 1,
            name: "apple".into(),
            price: 12.01,
            notes: Some("hand picked, organic".into()),
        }
        .into(),
    };

    let result = model.clone().into_active_model().insert(db).await?;

    assert_eq!(result, model);

    assert_eq!(
        Entity::find()
            .filter(Column::Id.eq(model.id))
            .one(db)
            .await?,
        Some(model)
    );

    Ok(())
}
