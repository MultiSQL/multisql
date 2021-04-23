use {
    super::AlterError,
    crate::{
        data::{get_name, schema::ColumnDefExt},
        macros::try_into,
        result::MutResult,
        store::{AlterTable, AutoIncrement, Store, StoreMut},
    },
    futures::stream::{self, TryStreamExt},
    sqlparser::ast::{ObjectName, ObjectType},
    std::fmt::Debug,
};

pub async fn drop<T: 'static + Debug, U: Store<T> + StoreMut<T> + AlterTable + AutoIncrement>(
    storage: U,
    object_type: &ObjectType,
    names: &[ObjectName],
    if_exists: bool,
) -> MutResult<U, ()> {
    if object_type != &ObjectType::Table {
        return Err((
            storage,
            AlterError::DropTypeNotSupported(object_type.to_string()).into(),
        ));
    }

    stream::iter(names.iter().map(Ok))
        .try_fold((storage, ()), |(storage, _), table_name| async move {
            let table_name = try_into!(storage, get_name(table_name));
            let schema = try_into!(storage, storage.fetch_schema(table_name).await);

            if !if_exists {
                try_into!(
                    storage,
                    schema
                        .clone()
                        .ok_or_else(|| AlterError::TableNotFound(table_name.to_owned()).into())
                );
            }

            #[cfg(feature = "auto-increment")]
            let (storage, _) = if let Some(schema) = schema {
                stream::iter(schema.column_defs.into_iter().map(Ok))
                    .try_fold((storage, ()), |(storage, _), column| async move {
                        if column.is_auto_incremented() {
                            storage
                                .set_increment_value(
                                    table_name,
                                    column.name.value.as_str(),
                                    0 as i64,
                                )
                                .await
                        } else {
                            Ok((storage, ()))
                        }
                    })
                    .await?
            } else {
                (storage, ())
            };

            storage.delete_schema(table_name).await
        })
        .await
}
