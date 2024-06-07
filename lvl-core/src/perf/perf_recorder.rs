use std::{collections::VecDeque, fmt::Display, time::Instant};

#[derive(Debug, Clone)]
pub struct PerfRecorder {
    name: String,
    current: Instant,
    update_times: VecDeque<f32>,
    late_update_times: VecDeque<f32>,
    prepare_render_times: VecDeque<f32>,
    render_times: VecDeque<f32>,
}

impl PerfRecorder {
    const MAX_FRAMES: usize = 32;

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            current: Instant::now(),
            update_times: VecDeque::with_capacity(Self::MAX_FRAMES),
            late_update_times: VecDeque::with_capacity(Self::MAX_FRAMES),
            prepare_render_times: VecDeque::with_capacity(Self::MAX_FRAMES),
            render_times: VecDeque::with_capacity(Self::MAX_FRAMES),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn frame_begin(&mut self) {
        self.current = Instant::now();
    }

    pub fn frame_update_end(&mut self) {
        let now = Instant::now();

        if Self::MAX_FRAMES <= self.update_times.len() {
            self.update_times.pop_front();
        }

        self.update_times
            .push_back((now - self.current).as_secs_f32());
        self.current = now;
    }

    pub fn frame_late_update_end(&mut self) {
        let now = Instant::now();

        if Self::MAX_FRAMES <= self.late_update_times.len() {
            self.late_update_times.pop_front();
        }

        self.late_update_times
            .push_back((now - self.current).as_secs_f32());
        self.current = now;
    }

    pub fn frame_prepare_render_end(&mut self) {
        let now = Instant::now();

        if Self::MAX_FRAMES <= self.prepare_render_times.len() {
            self.prepare_render_times.pop_front();
        }

        self.prepare_render_times
            .push_back((now - self.current).as_secs_f32());
        self.current = now;
    }

    pub fn frame_render_end(&mut self) {
        let now = Instant::now();

        if Self::MAX_FRAMES <= self.render_times.len() {
            self.render_times.pop_front();
        }

        self.render_times
            .push_back((now - self.current).as_secs_f32());
        self.current = now;
    }

    pub fn report(&self) -> PerfReport {
        let update_avg = self.update_times.iter().sum::<f32>() / self.update_times.len() as f32;
        let late_update_avg =
            self.late_update_times.iter().sum::<f32>() / self.late_update_times.len() as f32;
        let prepare_render_avg =
            self.prepare_render_times.iter().sum::<f32>() / self.prepare_render_times.len() as f32;
        let render_avg = self.render_times.iter().sum::<f32>() / self.render_times.len() as f32;

        PerfReport {
            name: &self.name,
            update_avg,
            late_update_avg,
            prepare_render_avg,
            render_avg,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerfReport<'a> {
    pub name: &'a str,
    pub update_avg: f32,
    pub late_update_avg: f32,
    pub prepare_render_avg: f32,
    pub render_avg: f32,
}

impl<'a> Display for PerfReport<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] update_avg: {:.2}ms, late_update_avg: {:.2}ms, prepare_render_avg: {:.2}ms, render_avg: {:.2}ms, total: {:.2}ms",
            self.name,
            self.update_avg * 1000.0,
            self.late_update_avg * 1000.0,
            self.prepare_render_avg * 1000.0,
            self.render_avg * 1000.0,
            (self.update_avg + self.late_update_avg + self.prepare_render_avg + self.render_avg) * 1000.0
        )
    }
}
