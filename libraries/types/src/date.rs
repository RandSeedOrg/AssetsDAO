use std::borrow::Cow;

use candid::{CandidType, Decode, Encode};
use chrono::{DateTime, Datelike};
use ic_stable_structures::{Storable, storable::Bound};
use serde::{Deserialize, Serialize};

use crate::TimestampNanos;

/// Nanoseconds in a day
pub const ONE_DAY_NANOS: u64 = 24 * 60 * 60 * 1_000_000_000;

/// Year, month, day format 20210101
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, CandidType, Default)]
pub struct YearMonthDay(u16, u8, u8);

impl YearMonthDay {
  pub fn new(year: u16, month: u8, day: u8) -> Self {
    Self(year, month, day)
  }

  pub fn year(&self) -> u16 {
    self.0
  }

  pub fn month(&self) -> u8 {
    self.1
  }

  pub fn day(&self) -> u8 {
    self.2
  }
}

/// Date range, (start date, end date)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, CandidType, Default)]
pub struct DateRange(YearMonthDay, YearMonthDay);

impl From<TimestampNanos> for YearMonthDay {
  fn from(time: TimestampNanos) -> Self {
    let datetime = DateTime::from_timestamp((time / 1_000_000_000) as i64, (time % 1_000_000_000) as u32).expect("invalid timestamp");
    YearMonthDay(datetime.year() as u16, datetime.month() as u8, datetime.day() as u8)
  }
}

impl DateRange {
  pub fn new(start: YearMonthDay, end: YearMonthDay) -> Self {
    Self(start, end)
  }

  pub fn contain_date(&self, date: YearMonthDay) -> bool {
    date >= self.0 && date <= self.1
  }

  pub fn contain_timestamp_nanos(&self, time: TimestampNanos) -> bool {
    let date = YearMonthDay::from(time);
    date >= self.0 && date <= self.1
  }

  pub fn start(&self) -> YearMonthDay {
    self.0
  }

  pub fn end(&self) -> YearMonthDay {
    self.1
  }
}

pub struct DateRangeIter<'a> {
  range: &'a DateRange,
  current: YearMonthDay,
}

pub struct DateRangeIntoIter {
  range: DateRange,
  current: YearMonthDay,
}

/// Get the maximum number of days in the month based on year and month
pub fn get_max_day(year: u16, month: u8) -> u8 {
  match month {
    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
    4 | 6 | 9 | 11 => 30,
    2 => {
      if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
        29
      } else {
        28
      }
    }
    _ => panic!("invalid month"),
  }
}

impl Iterator for DateRangeIter<'_> {
  type Item = YearMonthDay;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current <= self.range.1 {
      let current = self.current;
      self.current = match self.current {
        YearMonthDay(year, month, day) => {
          if day < get_max_day(year, month) {
            YearMonthDay(year, month, day + 1)
          } else if month < 12 {
            YearMonthDay(year, month + 1, 1)
          } else {
            YearMonthDay(year + 1, 1, 1)
          }
        }
      };
      Some(current)
    } else {
      None
    }
  }
}

impl Iterator for DateRangeIntoIter {
  type Item = YearMonthDay;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current <= self.range.1 {
      let current = self.current;
      self.current = match self.current {
        YearMonthDay(year, month, day) => {
          if day < get_max_day(year, month) {
            YearMonthDay(year, month, day + 1)
          } else if month < 12 {
            YearMonthDay(year, month + 1, 1)
          } else {
            YearMonthDay(year + 1, 1, 1)
          }
        }
      };
      Some(current)
    } else {
      None
    }
  }
}

impl<'a> IntoIterator for &'a DateRange {
  type Item = YearMonthDay;
  type IntoIter = DateRangeIter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    DateRangeIter {
      range: self,
      current: self.0,
    }
  }
}

impl IntoIterator for DateRange {
  type Item = YearMonthDay;
  type IntoIter = DateRangeIntoIter;

  fn into_iter(self) -> Self::IntoIter {
    DateRangeIntoIter {
      range: self,
      current: self.0,
    }
  }
}

impl Storable for YearMonthDay {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

impl Storable for DateRange {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}
