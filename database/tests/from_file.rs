
mod utils;
use crate::utils::*;

/// simple.db contains two projects: bar and baz
///
/// bar has two columns: col_1 and col_2
/// baz has two columns: col_1 and col_2
#[tokio::test]
#[allow(clippy::disallowed_names)]
async fn simple() {
    let db = create_file_db("tests/.read_files/simple.db".into()).await;

    let bar = get(&db, "bar").await.expect("Project bar should exist");
    let baz = get(&db, "baz").await.expect("Project baz should exist");

    let bar_columns = bar.get_columns().await.expect("Getting columns shouldn't fail");
    let baz_columns = baz.get_columns().await.expect("Getting columns shouldn't fail");

    assert_eq!(bar_columns.len(), 2);
    assert_eq!(baz_columns.len(), 2);

    assert_eq!(bar_columns[0].name, "col_1");
    assert_eq!(bar_columns[1].name, "col_2");

    assert_eq!(baz_columns[0].name, "col_1");
    assert_eq!(baz_columns[1].name, "col_2");
}
