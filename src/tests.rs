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

    #[test]
    fn push_1e1_iter_all() {
        let mut q = SerdeQueue::new();
        let n = 10;
        for i in 0..n {
            q.push(&i).unwrap();
        }
        assert_eq!(q.len(), n);
        let mut iter = q.iter();
        for i in 0..n {
            assert_eq!(iter.next(), Some(i));
        }
        assert_eq!(iter.next(), None);
    }
}
