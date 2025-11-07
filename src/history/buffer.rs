use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct DataPoint<T> {
    pub value: T,
    pub timestamp: u64,
}

impl<T> DataPoint<T> {
    pub fn new(value: T, timestamp: u64) -> Self {
        Self { value, timestamp }
    }
}

pub struct CircularBuffer<T> {
    data: VecDeque<DataPoint<T>>,
    capacity: usize,
}

impl<T: Clone> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: T, timestamp: u64) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(DataPoint::new(value, timestamp));
    }

    pub fn get_values(&self) -> Vec<T> {
        self.data.iter().map(|dp| dp.value.clone()).collect()
    }

    pub fn get_latest(&self) -> Option<&T> {
        self.data.back().map(|dp| &dp.value)
    }

    pub fn get_all(&self) -> &VecDeque<DataPoint<T>> {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn get_at(&self, index: usize) -> Option<(u64, T)> {
        self.data.get(index).map(|dp| (dp.timestamp, dp.value.clone()))
    }
}

impl<T: Clone> Default for CircularBuffer<T> {
    fn default() -> Self {
        Self::new(60)
    }
}

pub struct MetricsHistory {
    pub cpu_usage: CircularBuffer<f32>,
    pub memory_usage: CircularBuffer<f32>,
    pub swap_usage: CircularBuffer<f32>,
    pub gpu_temp: Vec<CircularBuffer<f32>>,
    pub gpu_util: Vec<CircularBuffer<f32>>,
    pub gpu_mem: Vec<CircularBuffer<f32>>,
}

impl MetricsHistory {
    pub fn new(capacity: usize, gpu_count: usize) -> Self {
        Self {
            cpu_usage: CircularBuffer::new(capacity),
            memory_usage: CircularBuffer::new(capacity),
            swap_usage: CircularBuffer::new(capacity),
            gpu_temp: (0..gpu_count).map(|_| CircularBuffer::new(capacity)).collect(),
            gpu_util: (0..gpu_count).map(|_| CircularBuffer::new(capacity)).collect(),
            gpu_mem: (0..gpu_count).map(|_| CircularBuffer::new(capacity)).collect(),
        }
    }

    pub fn update(&mut self, 
        timestamp: u64,
        cpu_usage: f32,
        memory_usage: f32,
        swap_usage: f32,
        gpu_infos: &[crate::gpu::GPUInfo],
    ) {
        self.cpu_usage.push(cpu_usage, timestamp);
        self.memory_usage.push(memory_usage, timestamp);
        self.swap_usage.push(swap_usage, timestamp);

        for (i, gpu_info) in gpu_infos.iter().enumerate() {
            if i < self.gpu_temp.len() {
                if let Some(temp) = gpu_info.temperature {
                    self.gpu_temp[i].push(temp, timestamp);
                }
                if let Some(util) = gpu_info.utilization {
                    self.gpu_util[i].push(util, timestamp);
                }
                if let Some(mem_percent) = gpu_info.memory_usage_percent() {
                    self.gpu_mem[i].push(mem_percent, timestamp);
                }
            }
        }
    }

    pub fn resize_gpu_buffers(&mut self, new_count: usize, capacity: usize) {
        while self.gpu_temp.len() < new_count {
            self.gpu_temp.push(CircularBuffer::new(capacity));
            self.gpu_util.push(CircularBuffer::new(capacity));
            self.gpu_mem.push(CircularBuffer::new(capacity));
        }
    }
}
