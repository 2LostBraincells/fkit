
mod utils;

mod stress_tests {
    use crate::utils::*;

    #[tokio::test]
    async fn projects_100() {
        let db = create_mem_db("projects_100").await;

        for i in 0..100 {
            cre_proj(&db, &format!("project_{}", i)).await;
        }

        let projects = get_all(&db).await;

        assert_eq!(projects.len(), 100);
    }

    #[tokio::test]
    async fn columns_100() {
        let db = create_mem_db("columns_100").await;

        let project = cre_proj(&db, "project").await;

        for i in 0..100 {
            cre_col(&project, &format!("column_{}", i))
                .await;
        }

        let columns = project.get_columns().await.expect("Getting columns shouldn't fail");
        assert_eq!(columns.len(), 100);
    }

    #[tokio::test]
    async fn matrix() {
        let db = create_mem_db("matrix").await;

        for i in 0..20 {
            let project = cre_proj(&db, &format!("project_{}", i)).await;

            for j in 0..20 {
                cre_col(&project, &format!("column_{}", j))
                    .await;
            }

            let columns = project.get_columns().await.expect("Getting columns shouldn't fail");
            assert_eq!(columns.len(), 20);
        }

        let projects = get_all(&db).await;
        assert_eq!(projects.len(), 20);
    }
}

mod read_file {
    use crate::utils::*;

    /// simple.db contains two projects: bar and baz
    ///
    /// bar has two columns: col_1 and col_2
    /// baz has two columns: col_1 and col_2
    #[tokio::test]
    #[allow(clippy::disallowed_names)]
    async fn simple_example() {
        let db = create_file_db("tests/simple.db".into()).await;

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
}
