
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
