#![allow(dead_code)]
#![allow(clippy::needless_range_loop)]

use std::collections::HashMap;
use std::ops::{Deref, Range};
use std::{
    ffi::c_int,
    sync::{Arc, Mutex},
    time::Duration,
};

use fontconfig::Fontconfig;
use rand::Rng;
use sdl2::rect::Rect;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color};

#[derive(Debug)]
enum MarkType {
    None,
    Read,
    Write,
    Pivot,
}

#[derive(Debug)]
struct Mark {
    ty: MarkType,
    frames: usize,
}

impl Mark {
    fn new(ty: MarkType) -> Self {
        Self { ty, frames: 0 }
    }
    fn set(&mut self, ty: MarkType) {
        self.ty = ty;
        self.frames = 0;
    }
    fn reset(&mut self) {
        self.ty = MarkType::None;
        self.frames = 0;
    }
}

#[derive(Debug)]
struct Value {
    value: c_int,
    mark: Mark,
}

impl Value {
    fn new(value: c_int) -> Self {
        Self {
            value,
            mark: Mark::new(MarkType::None),
        }
    }
}

#[derive(Clone, Debug)]
struct Context {
    phase_name: Arc<Mutex<String>>,
}

impl Context {
    fn set_phase(&self, phase: impl Into<String>) {
        *self.phase_name.lock().unwrap() = phase.into();
    }
}

const SORT_ELEMENTS: usize = 100;
const INSERTION_SORT_THRESHOLD: usize = 8;
const MARK_SHOWN_FRAMES: usize = 3;
const MEM_OP_DELAY: Duration = Duration::from_millis(10);

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let window = video_subsystem
        .window("visualize", 1500, 600)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let fontconfig = Fontconfig::new().unwrap();
    let font = fontconfig.find("sans-serif", None).unwrap();
    let font = ttf_context.load_font(font.path, 30).unwrap();
    let mut text_cache = HashMap::new();

    let new_array = || TargetArray {
        data: {
            let mut data = Vec::with_capacity(SORT_ELEMENTS);
            let mut rng = rand::thread_rng();

            for _ in 0..SORT_ELEMENTS {
                let v: f64 = rng.sample(rand_distr::Normal::new(2.0, 1.0).unwrap());
                let v = ((v * SORT_ELEMENTS as f64) / 4.0).clamp(0 as _, SORT_ELEMENTS as _);
                data.push(v as _);
            }

            let data = data.into_iter().map(Value::new).collect::<Vec<_>>();

            Arc::new(Mutex::new(data))
        },
        range: 0..SORT_ELEMENTS,
    };

    let context = Context {
        phase_name: Arc::new(Mutex::new(String::new())),
    };

    let mut target_array = new_array();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape | Keycode::Q),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    target_array = new_array();
                    let array = target_array.clone();
                    let context = context.clone();
                    std::thread::spawn(|| radixsort(array, context));
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let mut phase_name = context.phase_name.lock().unwrap().clone();
        if phase_name.is_empty() {
            phase_name = " ".into();
        }

        let phase_name = match text_cache.get(&phase_name) {
            Some(x) => x,
            None => {
                let surface = font
                    .render(&phase_name)
                    .blended(Color::RGB(255, 255, 255))
                    .unwrap();
                let texture = texture_creator
                    .create_texture_from_surface(&surface)
                    .unwrap();
                text_cache.insert(phase_name.clone(), texture);
                text_cache.get(&phase_name).unwrap()
            }
        };
        let query = phase_name.query();
        canvas
            .copy(
                phase_name,
                None,
                Some(Rect::new(0, 0, query.width as _, query.height as _)),
            )
            .unwrap();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        let (width, height) = canvas.output_size().unwrap();
        {
            let mut array = target_array.data.lock().unwrap();
            let bar_height_unit = height as f64 / SORT_ELEMENTS as f64;
            let bar_width = width / array.len() as u32;
            for (i, data) in array.iter_mut().enumerate() {
                let bar_color = match data.mark.ty {
                    MarkType::None => Color::RGB(255, 255, 255),
                    MarkType::Read => Color::RGB(0, 255, 0),
                    MarkType::Write => Color::RGB(255, 0, 0),
                    MarkType::Pivot => Color::RGB(255, 0, 255),
                };
                canvas.set_draw_color(bar_color);
                data.mark.frames += 1;
                if data.mark.frames >= MARK_SHOWN_FRAMES {
                    data.mark.reset();
                }
                let r = Rect::new(
                    bar_width as i32 * i as i32,
                    // TODO: SORT_ELEMENTS here should be (max - min) of array
                    (bar_height_unit * (SORT_ELEMENTS as i32 - data.value) as f64) as _,
                    bar_width,
                    height as _,
                );
                canvas.fill_rect(r).unwrap();
            }
        }

        canvas.present();
        std::thread::sleep(Duration::from_secs_f64(1.0 / 60.0));
    }
}

fn delay() {
    std::thread::sleep(MEM_OP_DELAY);
}

#[derive(Clone, Debug)]
struct TargetArray {
    data: Arc<Mutex<Vec<Value>>>,
    range: Range<usize>,
}

#[test]
fn e() {
    let t = TargetArray {
        data: Arc::new(Mutex::new(
            (1..=5).map(|x| Value::new(x as _)).collect::<Vec<_>>(),
        )),
        range: 0..5,
    };
    assert_eq!(t.len(), 5);

    let (a, b) = t.split_at(2);
    assert_eq!(a.range, 0..2);
    assert_eq!(a.len(), 2);

    assert_eq!(b.range, 2..5);
    assert_eq!(b.len(), 3);

    assert_eq!(a.get(0), 1);
    assert_eq!(a.get(1), 2);

    assert_eq!(b.get(0), 3);
    assert_eq!(b.get(1), 4);
    assert_eq!(b.get(2), 5);

    let (c, d) = b.split_at(1);
    assert_eq!(c.len(), 1);
    assert_eq!(c.range, 2..3);
    assert_eq!(d.len(), 2);
    assert_eq!(d.range, 3..5);
    assert_eq!(c.get(0), 3);
    assert_eq!(d.get(0), 4);
    assert_eq!(d.get(1), 5);
}

impl TargetArray {
    fn len(&self) -> usize {
        self.range.len()
    }

    fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    fn get(&self, index: usize) -> c_int {
        let ret = {
            let d = &mut self.data.lock().unwrap()[self.range()][index];
            d.mark.set(MarkType::Read);
            d.value
        };
        delay();
        ret
    }

    fn get_pivot(&self, index: usize) -> c_int {
        let ret = {
            let d = &mut self.data.lock().unwrap()[self.range()][index];
            d.mark.set(MarkType::Pivot);
            d.value
        };
        delay();
        ret
    }

    fn set(&self, index: usize, value: c_int) {
        {
            let d = &mut self.data.lock().unwrap()[self.range()][index];
            d.mark.set(MarkType::Write);
            d.value = value;
        }
        delay()
    }

    fn swap(&self, a: usize, b: usize) {
        {
            let mut lock = self.data.lock().unwrap();
            let tmp = lock[self.range()][a].value;
            lock[self.range()][a].value = lock[self.range()][b].value;
            lock[self.range()][b].value = tmp;
            lock[self.range()][a].mark.set(MarkType::Write);
            lock[self.range()][b].mark.set(MarkType::Write);
        }
        delay()
    }

    fn split_at(&self, mut index: usize) -> (Self, Self) {
        let start = self.range.start;
        let end = self.range.end;
        index += self.range.start;
        let a = TargetArray {
            data: self.data.clone(),
            range: start..index,
        };
        let b = TargetArray {
            data: self.data.clone(),
            range: index..end,
        };
        (a, b)
    }
}

struct PivotGuard {
    array: TargetArray,
    value: c_int,
    index: usize,
}

impl Deref for PivotGuard {
    type Target = c_int;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Drop for PivotGuard {
    fn drop(&mut self) {
        self.array.data.lock().unwrap()[self.index].mark.reset();
    }
}

fn insertion_sort(data: TargetArray) {
    for i in 1..data.len() {
        if data.get(i - 1) > data.get(i) {
            let mut slide_from = i;
            let sliding_value = data.get(slide_from);
            loop {
                data.set(slide_from, data.get(slide_from - 1));
                slide_from -= 1;
                if !(slide_from > 0 && data.get(slide_from - 1) > sliding_value) {
                    break;
                }
            }
            data.set(slide_from, sliding_value);
        }
    }
}

const PARTITION_BLOCK: usize = 128;

fn block_partition(data: &TargetArray, pivot: c_int, ctx: &Context) -> usize {
    let len = data.len();

    // ピボットより大きいか確認した場所までの index
    let mut left = 0;
    let mut left_start = 0;
    let mut left_len = 0;
    // ピボットより大きい数字の場所
    let mut left_offsets = [0; PARTITION_BLOCK];

    // ピボットより小さいか確認した場所までの index
    let mut right = len - 1;
    let mut right_start = 0;
    let mut right_len = 0;
    // ピボットより小さい数字の場所
    let mut right_offsets = [0; PARTITION_BLOCK];

    while right - left + 1 > 2 * PARTITION_BLOCK {
        if left_len == 0 {
            ctx.set_phase("Block Partition: correct left");
            left_start = 0;
            for i in 0..PARTITION_BLOCK {
                left_offsets[left_len] = i;
                left_len += (pivot < data.get(left + i)) as usize;
            }
        }
        if right_len == 0 {
            ctx.set_phase("Block Partition: correct right");
            right_start = 0;
            for i in 0..PARTITION_BLOCK {
                right_offsets[right_len] = i;
                right_len += (pivot > data.get(right - i)) as usize;
            }
        }

        ctx.set_phase("Block Partition: rearrangement");
        let num = left_len.min(right_len);
        for i in 0..num {
            data.swap(
                left + left_offsets[left_start + i],
                right - right_offsets[right_start + i],
            );
        }
        left_len -= num;
        right_len -= num;
        left_start += num;
        right_start += num;
        if left_len == 0 {
            left += PARTITION_BLOCK;
        }
        if right_len == 0 {
            right -= PARTITION_BLOCK;
        }
    }

    if right_len > 0 && left_len == 0 {
        // left_buffer に値がない
        // ピボットより大きい値が見つかっていない
        // leftを行けるところまで動かしてみて、その間にピボットより大きい値があれば right_buffer の値を使ってスワップする
        // この操作でright_bufferを使い切ればいつもの処理に持ち込める
        ctx.set_phase("Retain Right Buffer");
        'recovery: loop {
            if pivot < data.get(left) {
                data.swap(left, right - right_offsets[right_start]);
                right_len -= 1;
                right_start += 1;
                if right_len == 0 {
                    break 'recovery;
                }
            }
            if left >= right - right_offsets[right_start] {
                return left + 1;
            }
            left += 1;
        }
    }

    if left_len > 0 && right_len == 0 {
        ctx.set_phase("Retain Left Buffer");
        'recovery: loop {
            if pivot > data.get(right) {
                data.swap(right, left + left_offsets[left_start]);
                left_len -= 1;
                left_start += 1;
                if left_len == 0 {
                    break 'recovery;
                }
            }
            if left + left_offsets[left_start] >= right {
                return right;
            }
            right -= 1;
        }
    }

    loop {
        ctx.set_phase("Hoare Partition: Left");
        while data.get(left as _) < pivot {
            left += 1;
        }
        ctx.set_phase("Hoare Partition: Right");
        while data.get(right as _) > pivot {
            right -= 1;
        }
        if left >= right {
            break;
        }
        data.swap(left as _, right as _);
        left += 1;
        right -= 1;
    }

    left
}

fn quicksort(data: TargetArray, ctx: Context) {
    let len = data.len();

    if len <= 1 {
        return;
    }

    if len <= INSERTION_SORT_THRESHOLD {
        insertion_sort(data);
        return;
    }

    let mut rng = rand::thread_rng();
    let pivot = data.get(rng.gen_range(0..data.len()));
    let partition = block_partition(&data, pivot, &ctx);

    let (a, b) = data.split_at(partition as _);
    quicksort(a, ctx.clone());
    quicksort(b, ctx);
}

// copy-pasted from rust stdlib implementation
fn heapsort(data: TargetArray) {
    let sift_down = |v: &TargetArray, mut node| {
        loop {
            // Children of `node`.
            let mut child = 2 * node + 1;
            if child >= v.len() {
                break;
            }

            // Choose the greater child.
            if child + 1 < v.len() && v.get(child) < v.get(child + 1) {
                child += 1;
            }

            // Stop if the invariant holds at `node`.
            if v.get(node) >= v.get(child) {
                break;
            }

            // Swap `node` with the greater child, move one step down, and continue sifting.
            v.swap(node, child);
            node = child;
        }
    };

    let len = data.len();

    // Build the heap in linear time.
    for i in (0..len / 2).rev() {
        sift_down(&data, i);
    }

    // Pop maximal elements from the heap.
    for i in (1..len).rev() {
        data.swap(0, i);
        let (a, _) = data.split_at(i);
        sift_down(&a, 0);
    }
}

fn radixsort(data: TargetArray, _context: Context) {
    let mut temp = vec![0; data.len()];

    let int_bits = c_int::BITS;
    let mut max_bits = 0u32;
    for i in 0..data.len() {
        let value = data.get(i);
        for bit in 0..int_bits {
            if (value & (1 << bit)) != 0 {
                max_bits = max_bits.max(bit);
            }
        }
    }

    for bit in 0..=max_bits {
        let mut counter = 0;
        for i in 0..data.len() {
            let value = data.get(i);
            temp[i] = value;
            if (value & (1 << bit)) == 0 {
                counter += 1;
            }
        }
        let mut index = [0, counter];
        for i in 0..data.len() {
            if (temp[i] & (1 << bit)) == 0 {
                data.set(index[0], temp[i]);
                index[0] += 1;
            } else {
                data.set(index[1], temp[i]);
                index[1] += 1;
            }
        }
    }
}
