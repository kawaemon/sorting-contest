#![allow(dead_code)]
use std::collections::HashMap;
use std::ops::{Deref, Range};
use std::{
    ffi::c_int,
    sync::{Arc, Mutex},
    time::Duration,
};

use fontconfig::Fontconfig;
use rand::distributions::Standard;
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

const SORT_ELEMENTS: usize = 3000;
const MARK_SHOWN_FRAMES: usize = 5;
const MEM_OP_DELAY: Duration = Duration::from_nanos(1000000);

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
    let mut font = ttf_context.load_font(font.path, 128).unwrap();
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

            let data = data.into_iter().map(|x| Value::new(x)).collect::<Vec<_>>();

            Arc::new(Mutex::new(data))
        },
        range: 0..SORT_ELEMENTS,
    };

    let mut context = Context {
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
                    std::thread::spawn(|| quicksort(array, context));
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
            let bar_height_unit = height as f64 / SORT_ELEMENTS as f64; // TODO:  constant
            let bar_width = width / array.len() as u32;
            for (i, data) in array.iter_mut().enumerate() {
                match data.mark.ty {
                    MarkType::None => canvas.set_draw_color(Color::RGB(255, 255, 255)),
                    MarkType::Read => canvas.set_draw_color(Color::RGB(0, 255, 0)),
                    MarkType::Write => canvas.set_draw_color(Color::RGB(255, 0, 0)),
                    MarkType::Pivot => canvas.set_draw_color(Color::RGB(255, 0, 255)),
                }
                data.mark.frames += 1;
                if data.mark.frames >= MARK_SHOWN_FRAMES {
                    data.mark.reset();
                }
                let r = Rect::new(
                    bar_width as i32 * i as i32,
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

    ctx.set_phase("Block Partition");
    while right - left + 1 > 2 * PARTITION_BLOCK {
        if left_len == 0 {
            left_start = 0;
            for i in 0..PARTITION_BLOCK {
                left_offsets[left_len] = i;
                left_len += (pivot < data.get(left + i)) as usize;
            }
        }
        if right_len == 0 {
            right_start = 0;
            for i in 0..PARTITION_BLOCK {
                right_offsets[right_len] = i;
                right_len += (pivot > data.get(right - i)) as usize;
            }
        }

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
        println!("retaining right_buffer");
        ctx.set_phase("Retain Right Buffer");
        'recovery: loop {
            if pivot < data.get(left) {
                data.swap(left, right - right_offsets[right_start]);
                right_len -= 1;
                right_start += 1;
                if right_len == 0 {
                    println!("used full of right_buffer");
                    break 'recovery;
                }
            }
            if left >= right - right_offsets[right_start] {
                println!("crossover");
                return left + 1;
            }
            left += 1;
        }
    }

    if left_len > 0 && right_len == 0 {
        println!("retaining left_buffer");
        ctx.set_phase("Retain Left Buffer");
        'recovery: loop {
            if pivot > data.get(right) {
                data.swap(right, left + left_offsets[left_start]);
                left_len -= 1;
                left_start += 1;
                if left_len == 0 {
                    println!("used full of left_buffer");
                    break 'recovery;
                }
            }
            if left + left_offsets[left_start] >= right {
                println!("crossover");
                return right;
            }
            right -= 1;
        }
    }

    ctx.set_phase("Hoare Partition");
    loop {
        while data.get(left as _) < pivot {
            left += 1;
        }
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

    if len <= 8 {
        insertion_sort(data);
        return;
    }

    let pivot = {
        let lo = 0;
        let mid = len / 2;
        let hi = len - 1;
        if data.get(mid) < data.get(lo) {
            data.swap(lo, mid)
        }
        if data.get(hi) < data.get(lo) {
            data.swap(hi, lo);
        }
        if data.get(mid) < data.get(hi) {
            data.swap(mid, hi);
        }
        data.get(hi)
    };

    let partition = block_partition(&data, pivot, &ctx);

    let (a, b) = data.split_at(partition as _);
    quicksort(a, ctx.clone());
    quicksort(b, ctx);
}
