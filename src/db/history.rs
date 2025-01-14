use std::fmt;

use std::collections::HashMap;

use rand::{distributions::{Distribution, Bernoulli, Uniform}, random, Rng};

use super::distribution::MyDistributionTrait;

use chrono::{DateTime, Duration, Local};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Event {
    pub write: bool,
    pub variable: usize,
    pub value: usize,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Transaction {
    pub events: Vec<Event>,
    pub success: bool,
}

pub type Session = Vec<Transaction>;

#[derive(Clone, Copy)]
pub struct HistoryParams<'a> {
    pub n_hist: usize,
    pub n_node: usize,
    pub n_variable: usize,
    pub n_transaction: usize,
    pub n_event: usize,
    pub read_probability: f64,
    pub longtxn_proportion: f64,
    pub longtxn_size: f64,
    pub key_distribution: &'a dyn MyDistributionTrait,
    pub random_txn_size: bool,
    pub repeated_value_proportion: f64,
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repr = format!(
            "<{}({}):{:2}>",
            if self.write { 'W' } else { 'R' },
            self.variable,
            self.value
        );
        if !self.success {
            write!(f, "!")?;
        }
        write!(f, "{}", repr)
    }
}

impl Event {
    pub fn read(var: usize) -> Self {
        Event {
            write: false,
            variable: var,
            value: 0,
            success: false,
        }
    }
    pub fn write(var: usize, val: usize) -> Self {
        Event {
            write: true,
            variable: var,
            value: val,
            success: false,
        }
    }
}

impl fmt::Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repr = format!("{:?}", self.events);
        if !self.success {
            write!(f, "!")?;
        }
        write!(f, "{}", repr)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HistParams {
    id: usize,
    n_node: usize,
    n_variable: usize,
    n_transaction: usize,
    n_event: usize,
}

impl HistParams {
    pub fn get_id(&self) -> usize {
        self.id
    }
    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }
    pub fn get_n_node(&self) -> usize {
        self.n_node
    }
    pub fn get_n_variable(&self) -> usize {
        self.n_variable
    }
    pub fn get_n_transaction(&self) -> usize {
        self.n_transaction
    }
    pub fn get_event(&self) -> usize {
        self.n_event
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct History {
    params: HistParams,
    info: String,
    start: DateTime<Local>,
    end: DateTime<Local>,
    data: Vec<Session>,
}

impl History {
    pub fn new(
        params: HistParams,
        info: String,
        start: DateTime<Local>,
        end: DateTime<Local>,
        data: Vec<Session>,
    ) -> Self {
        History {
            params,
            info,
            start,
            end,
            data,
        }
    }

    pub fn get_id(&self) -> usize {
        self.params.get_id()
    }

    pub fn get_data(&self) -> &Vec<Session> {
        &self.data
    }

    pub fn get_cloned_data(&self) -> Vec<Session> {
        self.data.clone()
    }

    pub fn get_params(&self) -> &HistParams {
        &self.params
    }

    pub fn get_cloned_params(&self) -> HistParams {
        self.params.clone()
    }

    pub fn get_duration(&self) -> Duration {
        self.end - self.start
    }
}

pub fn generate_single_history(
    params: HistoryParams
) -> Vec<Session> {
    let mut counters = HashMap::new();
    let mut random_generator = rand::thread_rng();
    let read_distribution = Bernoulli::new(params.read_probability).unwrap();
    let longtxn_distribution = Bernoulli::new(params.longtxn_proportion).unwrap();
    let shorttxn_size_distribution = Uniform::new(1, 2 * params.n_event - 1);
    let longtxn_size_distribution = Uniform::new((params.n_event as f64 * params.longtxn_size * 0.5) as usize,
                                                 (params.n_event as f64 * params.longtxn_size * 1.5) as usize);
    // let _jump = (params.n_variable as f64 / params.n_node as f64).ceil() as usize;
    (0..params.n_node).map(|_| {
        // let i = i_node * jump;
        // let j = std::cmp::min((i_node + 1) * jump, n_variable);
        // let write_variable_range = Uniform::from(i..j);
        (0..params.n_transaction).map(|_| {
            let size = if params.random_txn_size {
                if longtxn_distribution.sample(&mut random_generator) {
                    longtxn_size_distribution.sample(&mut random_generator)
                } else {
                    shorttxn_size_distribution.sample(&mut random_generator)
                }
            } else {
                if longtxn_distribution.sample(&mut random_generator) {
                    (params.n_event as f64 * params.longtxn_size) as usize
                } else {
                    params.n_event
                }
            };

            let generate_event = |_| {
                if read_distribution.sample(&mut random_generator) {
                    let variable = params.key_distribution.sample(&mut random_generator);
                    Event::read(variable)
                } else {
                    let variable = params.key_distribution.sample(&mut random_generator);
                    // let variable = write_variable_range.sample(&mut random_generator);
                    // let value = {
                    //     let entry = counters.entry(variable).or_insert(0);
                    //     *entry += 1;
                    //     *entry
                    // };
                    let value = if counters.contains_key(&variable) {
                        if !random_generator.gen_bool(params.repeated_value_proportion) {
                            let entry = counters.entry(variable).or_insert(0);
                            *entry += 1;
                            *entry
                        } else {
                            let range = 1..=counters[&variable];
                            random_generator.gen_range(range)
                        }
                    } else {
                        let entry = counters.entry(variable).or_insert(0);
                        *entry += 1;
                        *entry
                    };

                    Event::write(variable, value)
                }
            };

            Transaction {
                events: (0..size).map(generate_event).collect(),
                success: false,
            }
        }).collect()
    }).collect()
}

pub fn generate_mult_histories(
    params: HistoryParams
) -> Vec<History> {
    (0..params.n_hist).map(|i_hist| -> History {
        let start_time = Local::now();
        let hist = generate_single_history(
            params
        );
        let end_time = Local::now();
        History {
            params: HistParams {
                id: i_hist,
                n_node: params.n_node,
                n_variable: params.n_variable,
                n_transaction: params.n_transaction,
                n_event: params.n_event,
            },
            info: "generated".to_string(),
            start: start_time,
            end: end_time,
            data: hist,
        }
    }).collect()
}
