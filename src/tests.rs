#[cfg(test)]
mod tests {
    use crate::SerdeQueue;

    #[test]
    fn push_1_pop_1() {
        let x: usize = 233;
        let mut q = SerdeQueue::new();
        q.push(&x).unwrap();
        assert_eq!(q.pop().unwrap(), Some(x));
    }
}
