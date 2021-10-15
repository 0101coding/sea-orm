pub mod common;

pub use common::{bakery_chain::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use sea_orm::{entity::prelude::*, DatabaseConnection, IntoActiveModel, QueryOrder};

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() -> Result<(), DbErr> {
    let ctx = TestContext::new("bakery_chain_schema_self_join_tests").await;
    create_metadata(&ctx.db).await?;
    ctx.delete().await;

    Ok(())
}

pub async fn create_metadata(db: &DatabaseConnection) -> Result<(), DbErr> {
    let model = metadata::Model {
        uuid: Uuid::new_v4(),
        uuid_ref: None,
        ty: "Type".to_owned(),
        key: "markup".to_owned(),
        value: "1.18".to_owned(),
        bytes: vec![1, 2, 3],
        date: Some(Date::from_ymd(2021, 9, 27)),
        time: Some(Time::from_hms(1, 00, 00)),
    };

    model.clone().into_active_model().insert(db).await?;

    let linked_model = metadata::Model {
        uuid: Uuid::new_v4(),
        uuid_ref: Some(model.clone().uuid),
        time: Some(Time::from_hms(2, 00, 00)),
        ..model.clone()
    };

    linked_model.clone().into_active_model().insert(db).await?;

    let not_linked_model = metadata::Model {
        uuid: Uuid::new_v4(),
        time: Some(Time::from_hms(3, 00, 00)),
        ..model.clone()
    };

    not_linked_model
        .clone()
        .into_active_model()
        .insert(db)
        .await?;

    assert_eq!(
        model
            .find_linked(metadata::SelfReferencingLink)
            .all(db)
            .await?,
        vec![]
    );

    assert_eq!(
        linked_model
            .find_linked(metadata::SelfReferencingLink)
            .all(db)
            .await?,
        vec![model.clone()]
    );

    assert_eq!(
        not_linked_model
            .find_linked(metadata::SelfReferencingLink)
            .all(db)
            .await?,
        vec![]
    );

    assert_eq!(
        metadata::Entity::find()
            .find_also_linked(metadata::SelfReferencingLink)
            .order_by_asc(metadata::Column::Time)
            .all(db)
            .await?,
        vec![
            (model.clone(), None),
            (linked_model, Some(model)),
            (not_linked_model, None),
        ]
    );

    Ok(())
}
