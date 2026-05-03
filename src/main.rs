mod index;

fn main() {
    let mut vector_index = index::vector::vector::Index::new(2);
    vector_index
        .insert(index::vector::entity::Point::new(
            "1".to_string(),
            vec![1.0, 1.0],
        ))
        .unwrap();

    vector_index
        .insert(index::vector::entity::Point::new(
            "2".to_string(),
            vec![10.0, 10.0],
        ))
        .unwrap();

    vector_index
        .insert(index::vector::entity::Point::new(
            "2".to_string(),
            vec![20.0, 20.0],
        ))
        .unwrap();

    let result = vector_index
        .search(index::vector::param::SearchParam::new(vec![14.0, 14.0], 5))
        .unwrap();

    for point in result.result {
        println!("{}:{:?}", point.id, point.pos);
    }
}
