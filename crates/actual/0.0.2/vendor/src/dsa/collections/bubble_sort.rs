pub fn bubble_sort<T, C>(arr: &mut C)
where
    T: PartialOrd,
    C: AsMut<[T]>,
{
    let arr = arr.as_mut();
    let n = arr.len();

    // let l: usize = 01
    for i in 0..n {
        for j in 0..n - i - 1 {
            if arr[j] > arr[j + 1] {
                arr.swap(j, j + 1);
            }
        }
    }
}
