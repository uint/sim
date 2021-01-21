use std::any::Any;
use std::f64::INFINITY;

use serde::{Deserialize, Serialize};

use super::model::Model;
use super::ModelMessage;
use crate::input_modeling::random_variable::RandomVariable;
use crate::input_modeling::uniform_rng::UniformRNG;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Processor {
    id: String,
    service_time: RandomVariable,
    #[serde(default = "max_usize")]
    queue_capacity: usize,
    #[serde(default)]
    metrics_output: bool,
    ports_in: PortsIn,
    ports_out: PortsOut,
    #[serde(default)]
    state: State,
    #[serde(default)]
    snapshot: Metrics,
    #[serde(default)]
    history: Vec<Metrics>,
}

fn max_usize() -> usize {
    usize::MAX
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortsIn {
    job: String,
    snapshot: Option<String>,
    history: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PortsOut {
    processed_job: String,
    snapshot: Option<String>,
    history: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct State {
    event_list: Vec<ScheduledEvent>,
    until_job_completion: f64,
    queue: Vec<String>,
    phase: Phase,
    #[serde(default)]
    global_time: f64,
}

impl Default for State {
    fn default() -> Self {
        let initalization_event = ScheduledEvent {
            time: 0.0,
            event: Event::Run,
        };
        State {
            event_list: vec![initalization_event],
            until_job_completion: INFINITY,
            queue: Vec::new(),
            phase: Phase::Passive,
            global_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Phase {
    Active,
    Passive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Event {
    Run,
    DropJob,
    BeginProcessing,
    SendJob,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScheduledEvent {
    time: f64,
    event: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metrics {
    queue_size: usize,
    last_arrival: Option<(String, f64)>,
    last_service_start: Option<(String, f64)>,
    last_completion: Option<(String, f64)>,
    is_utilized: bool,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            queue_size: 0,
            last_arrival: None,
            last_service_start: None,
            last_completion: None,
            is_utilized: false,
        }
    }
}

impl Processor {
    fn need_snapshot_metrics(&self) -> bool {
        self.ports_in.snapshot.is_some() && self.ports_out.snapshot.is_some()
    }

    fn need_historical_metrics(&self) -> bool {
        self.need_snapshot_metrics()
            && self.ports_in.history.is_some()
            && self.ports_out.history.is_some()
    }
}

impl Model for Processor {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn status(&self) -> String {
        match self.state.phase {
            Phase::Active => String::from("Processing"),
            Phase::Passive => String::from("Passive"),
        }
    }

    fn events_ext(
        &mut self,
        _uniform_rng: &mut UniformRNG,
        incoming_message: ModelMessage,
    ) -> Vec<ModelMessage> {
        let mut outgoing_messages: Vec<ModelMessage> = Vec::new();
        let incoming_port: String = incoming_message.port_name;
        match &self.ports_in {
            PortsIn { job, .. } if *job == incoming_port => {
                self.state.queue.push(incoming_message.message.clone());
                // Possible metrics updates
                if self.need_snapshot_metrics() {
                    self.snapshot.queue_size = self.state.queue.len();
                    self.snapshot.last_arrival =
                        Some((incoming_message.message, self.state.global_time));
                }
                if self.need_historical_metrics() {
                    self.history.push(self.snapshot.clone());
                }
                // Possible subsequent event scheduling
                match (
                    self.state.queue.len() > self.queue_capacity,
                    self.state.phase.clone(),
                ) {
                    (true, _) => {
                        // Immediately drop the job that exceeded the queue capacity
                        self.state.event_list.push(ScheduledEvent {
                            time: 0.0,
                            event: Event::DropJob,
                        })
                    }
                    (false, Phase::Passive) => {
                        // Begin processing - there are now jobs to process
                        self.state.event_list.push(ScheduledEvent {
                            time: 0.0,
                            event: Event::BeginProcessing,
                        })
                    }
                    (false, Phase::Active) => {
                        // Nothing to do here - continue with existing processing schedule
                    }
                }
            }
            PortsIn { snapshot, .. } if Some(incoming_port.clone()) == *snapshot => {
                outgoing_messages.push(ModelMessage {
                    port_name: self.ports_out.snapshot.clone().unwrap(),
                    message: serde_json::to_string(&self.snapshot).unwrap(),
                });
            }
            PortsIn { history, .. } if Some(incoming_port) == *history => {
                outgoing_messages.push(ModelMessage {
                    port_name: self.ports_out.history.clone().unwrap(),
                    message: serde_json::to_string(&self.history).unwrap(),
                });
            }
            _ => panic!["ModelMessage recieved on a non-existent port"],
        };
        outgoing_messages
    }

    fn events_int(&mut self, uniform_rng: &mut UniformRNG) -> Vec<ModelMessage> {
        let mut outgoing_messages: Vec<ModelMessage> = Vec::new();
        let events = self.state.event_list.clone();
        self.state.event_list = self
            .state
            .event_list
            .iter()
            .filter(|scheduled_event| scheduled_event.time != 0.0)
            .cloned()
            .collect();
        events
            .iter()
            .filter(|scheduled_event| scheduled_event.time == 0.0)
            .for_each(|scheduled_event| match scheduled_event.event {
                Event::Run => {
                    if self.need_snapshot_metrics() {
                        self.snapshot = Metrics::default();
                    }
                    if self.need_historical_metrics() {
                        self.history.push(Metrics::default());
                    }
                }
                Event::DropJob => {
                    self.state.queue.remove(self.state.queue.len() - 1);
                    if self.need_snapshot_metrics() {
                        self.snapshot.queue_size = self.state.queue.len();
                    }
                    if self.need_historical_metrics() {
                        self.history.push(self.snapshot.clone());
                    }
                }
                Event::BeginProcessing => {
                    self.state.until_job_completion = self.service_time.random_variate(uniform_rng);
                    self.state.phase = Phase::Active;
                    if self.need_snapshot_metrics() {
                        self.snapshot.last_service_start = Some((
                            self.state.queue.first().unwrap().to_string(),
                            self.state.global_time,
                        ));
                        self.snapshot.is_utilized = true;
                    }
                    if self.need_historical_metrics() {
                        self.history.push(self.snapshot.clone());
                    }
                    self.state.event_list.push(ScheduledEvent {
                        time: self.state.until_job_completion,
                        event: Event::SendJob,
                    });
                }
                Event::SendJob => {
                    if self.need_snapshot_metrics() {
                        self.snapshot.last_completion = Some((
                            self.state.queue.first().unwrap().to_string(),
                            self.state.global_time,
                        ));
                    }
                    // Use just the job ID from the input message - transform job type
                    outgoing_messages.push(ModelMessage {
                        port_name: self.ports_out.processed_job.clone(),
                        message: format![
                            "{job_type} {job_id}",
                            job_type = self.ports_out.processed_job,
                            job_id = self.state.queue.remove(0).split(' ').last().unwrap()
                        ],
                    });
                    self.state.phase = Phase::Passive;
                    if self.need_snapshot_metrics() {
                        self.snapshot.is_utilized = false;
                        self.snapshot.queue_size = self.state.queue.len();
                    }
                    if self.need_historical_metrics() {
                        self.history.push(self.snapshot.clone());
                    }
                    if !self.state.queue.is_empty() {
                        self.state.event_list.push(ScheduledEvent {
                            time: 0.0,
                            event: Event::BeginProcessing,
                        });
                    }
                }
            });
        outgoing_messages
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.state
            .event_list
            .iter_mut()
            .for_each(|scheduled_event| {
                scheduled_event.time -= time_delta;
            });
        self.state.global_time += time_delta;
    }

    fn until_next_event(&self) -> f64 {
        self.state
            .event_list
            .iter()
            .fold(INFINITY, |until_next_event, event| {
                f64::min(until_next_event, event.time)
            })
    }
}
