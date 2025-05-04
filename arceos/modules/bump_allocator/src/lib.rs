#![no_std]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;
 


#[macro_use]
extern crate log;
/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
///
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    /// 内存范围起始地址
    start: usize,
    /// 内存范围结束地址
    end: usize,
    /// 字节分配当前位置（前向移动）
    b_pos: usize,
    /// 页分配当前位置（后向移动）
    p_pos: usize,
    /// 分配计数器
    count: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    /// 创建一个新的早期分配器
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }

    /// 对齐地址到指定对齐要求
    #[inline]
    fn align_up(&self, addr: usize, align: usize) -> usize {
        (addr + align - 1) & !(align - 1)
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, end: usize) {
        let mut start = 0xffffffc08024d000;
        let mut end = 0xffffffc088000000;
        debug!("EarlyAllocator init: [{:#x}, {:#x})", start, end);

        assert!(start < end, "Invalid memory range");
        self.start = start;
        self.end = end;
        self.b_pos = start;
        self.p_pos = end;
        self.count = 0;
    }

    fn add_memory(&mut self, _start: usize, _end: usize) -> AllocResult {
        Err(AllocError::InvalidParam)
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();

        if size == 0 {
            return Err(AllocError::InvalidParam);
        }

        let aligned_pos = self.align_up(self.b_pos, align);
        let new_pos = aligned_pos + size;

        debug!(
            "Allocating {} bytes, aligned position: {:#x}, new position: {:#x}, b_pos: {:#x}, p_pos: {:#x}",
            size, aligned_pos, new_pos, self.b_pos, self.p_pos
        );

        if new_pos > self.p_pos {
            return Err(AllocError::NoMemory);
        }

        self.b_pos = new_pos;
        self.count += 1;

        NonNull::new(aligned_pos as *mut u8).ok_or(AllocError::InvalidParam)
    }

    fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
        if self.count > 0 {
            self.count -= 1;
            if self.count == 0 {
                self.b_pos = self.start;
            }
        }
    }

    fn total_bytes(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    fn used_bytes(&self) -> usize {
        self.b_pos.saturating_sub(self.start)
    }

    fn available_bytes(&self) -> usize {
        if self.p_pos > self.b_pos {
            self.p_pos - self.b_pos
        } else {
            0
        }
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, pages: usize, align_pow2: usize) -> AllocResult<usize> {
        if pages == 0 {
            return Err(AllocError::InvalidParam);
        }

        let size = pages * Self::PAGE_SIZE;
        let new_pos = self.p_pos.saturating_sub(size);
        let aligned_pos = new_pos & !(align_pow2 - 1);

        debug!(
            "Allocating {} bytes, aligned position: {:#x}, new position: {:#x}, b_pos: {:#x}, p_pos: {:#x}",
            size, aligned_pos, new_pos, self.b_pos, self.p_pos
        );

        if aligned_pos < self.b_pos || aligned_pos > self.p_pos {
            return Err(AllocError::NoMemory);
        }

        self.p_pos = aligned_pos;
        Ok(aligned_pos)
    }

    fn dealloc_pages(&mut self, _pos: usize, _pages: usize) {
        // 页面区域不会被释放
    }

    fn total_pages(&self) -> usize {
        self.total_bytes() / Self::PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end.saturating_sub(self.p_pos)) / Self::PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        if self.p_pos > self.b_pos {
            (self.p_pos - self.b_pos) / Self::PAGE_SIZE
        } else {
            0
        }
    }
}
