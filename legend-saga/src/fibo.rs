pub(crate) fn fibonacci(n: usize) -> usize {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0;
            let mut b = 1;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

#[cfg(test)]
mod test_fibo {
    use crate::fibo::fibonacci;

    #[test]
    fn test_fibonacci() {
        // Test base cases
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);

        // Test first few Fibonacci numbers
        assert_eq!(fibonacci(2), 1);
        assert_eq!(fibonacci(3), 2);
        assert_eq!(fibonacci(4), 3);
        assert_eq!(fibonacci(5), 5);
        assert_eq!(fibonacci(6), 8);
        assert_eq!(fibonacci(7), 13);

        // Test a larger number
        assert_eq!(fibonacci(10), 55);

        // Test an even larger number
        assert_eq!(fibonacci(20), 6765);
    }

    #[test]
    fn test_fibonacci_sequence() {
        let expected = vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34];
        let actual: Vec<usize> = (0..10).map(fibonacci).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_fibonacci_large_input() {
        // This test checks that the function can handle larger inputs without overflowing
        // Note: The actual value might be different depending on the size of usize on your system
        let result = fibonacci(50);
        assert_eq!(result, 12586269025);
    }
}