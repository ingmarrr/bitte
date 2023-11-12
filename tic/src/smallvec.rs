use std::{
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Index, IndexMut},
};

pub union Storage<T, const N: usize>
where
    T: Sized + Copy,
{
    inline: [MaybeUninit<T>; N],
    heap: ManuallyDrop<Vec<T>>,
}

pub struct SmallVec<T, const N: usize>
where
    T: Sized + Copy,
{
    storage: Storage<T, N>,
    is_heap: bool,
    len: usize,
}

impl<T, const N: usize> Clone for SmallVec<T, N>
where
    T: Sized + Copy,
{
    fn clone(&self) -> Self {
        let mut new_vector = SmallVec::<T, N>::new();

        if self.is_heap {
            new_vector.storage = Storage {
                heap: unsafe { std::ptr::read(&self.storage.heap).clone() },
            };
            new_vector.is_heap = true;
        } else {
            for i in 0..self.len {
                unsafe {
                    let item = std::ptr::read(self.storage.inline.as_ptr().add(i).cast::<T>());
                    std::ptr::write(
                        new_vector.storage.inline.as_mut_ptr().add(i),
                        MaybeUninit::new(item),
                    );
                }
            }
        }

        new_vector.len = self.len;
        new_vector
    }
}

impl<T, const N: usize> SmallVec<T, N>
where
    T: Sized + Copy,
{
    pub fn new() -> Self {
        Self {
            storage: Storage {
                inline: unsafe { MaybeUninit::uninit().assume_init() },
            },
            is_heap: false,
            len: 0,
        }
    }

    pub fn push(&mut self, el: T) {
        if self.len < N {
            unsafe {
                std::ptr::write(
                    self.storage.inline.as_mut_ptr().add(self.len),
                    MaybeUninit::new(el),
                );
            }
        } else {
            if !self.is_heap {
                let mut heap = Vec::with_capacity(N * 2);
                unsafe {
                    for i in 0..N {
                        heap.push(
                            std::ptr::read(self.storage.inline.as_ptr().add(i)).assume_init(),
                        );
                    }
                }
                self.storage.heap = ManuallyDrop::new(heap);
                self.is_heap = true;
            }
        }
        self.len += 1;
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        if self.is_heap {
            unsafe { Some(&self.storage.heap[index]) }
        } else {
            unsafe { Some(&*self.storage.inline.as_ptr().add(index).cast::<T>()) }
        }
    }

    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.len, "index out of bounds");

        let result;
        if self.is_heap {
            result = unsafe { std::ptr::read(self.storage.heap.as_ptr().add(index)) };
            unsafe {
                (&mut self.storage.heap).remove(index);
            }
        } else {
            result = unsafe { std::ptr::read(self.storage.inline.as_ptr().add(index).cast::<T>()) };
            for i in index..self.len - 1 {
                unsafe {
                    let src = self.storage.inline.as_ptr().add(i + 1).cast::<T>();
                    let dst = self.storage.inline.as_mut_ptr().add(i).cast::<T>();
                    std::ptr::copy(src, dst, 1);
                }
            }
        }
        self.len -= 1;
        result
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T, const N: usize> Index<usize> for SmallVec<T, N>
where
    T: Sized + Copy,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<T, const N: usize> IndexMut<usize> for SmallVec<T, N>
where
    T: Sized + Copy,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len, "index out of bounds");
        if self.is_heap {
            unsafe { &mut (&mut self.storage.heap)[index] }
        } else {
            unsafe { &mut *self.storage.inline.as_mut_ptr().add(index).cast::<T>() }
        }
    }
}

impl<T, const N: usize> Drop for SmallVec<T, N>
where
    T: Sized + Copy,
{
    fn drop(&mut self) {
        if self.is_heap {
            unsafe {
                ManuallyDrop::drop(&mut self.storage.heap);
            }
        } else {
            unsafe {
                for i in 0..self.len {
                    std::ptr::drop_in_place(self.storage.inline.as_mut_ptr().add(i));
                }
            }
        }
    }
}
