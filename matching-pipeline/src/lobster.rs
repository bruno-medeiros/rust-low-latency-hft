use crate::command::OrderCommand;
use limit_order_book::types::{OrderId, Price, Qty, Side};

/// LOBSTER event types (values match the CSV encoding).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LobsterEventType {
    NewOrder = 1,
    PartialCancel = 2,
    FullDelete = 3,
    VisibleExecution = 4,
    HiddenExecution = 5,
    CrossTrade = 6,
    TradingHalt = 7,
}

/// A single parsed row from a LOBSTER message file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LobsterRow {
    pub timestamp_ns: u64,
    pub event_type: LobsterEventType,
    pub order_id: OrderId,
    pub size: Qty,
    pub price: Price,
    pub direction: Side,
}

#[derive(Debug, thiserror::Error)]
pub enum LobsterParseError {
    #[error("expected 6 CSV columns, got {0}")]
    WrongColumnCount(usize),

    #[error("invalid event type: {0}")]
    InvalidEventType(String),

    #[error("invalid direction: {0} (expected 1 or -1)")]
    InvalidDirection(String),

    #[error("failed to parse field '{field}': {source}")]
    FieldParse {
        field: &'static str,
        source: std::num::ParseIntError,
    },

    #[error("failed to parse timestamp '{0}'")]
    TimestampParse(String),
}

impl LobsterRow {
    /// Parse a single CSV line in LOBSTER message format.
    pub fn parse(line: &str) -> Result<Self, LobsterParseError> {
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() != 6 {
            return Err(LobsterParseError::WrongColumnCount(cols.len()));
        }

        Ok(Self {
            timestamp_ns: parse_timestamp(cols[0].trim())?,
            event_type: parse_event_type(cols[1].trim())?,
            order_id: cols[2]
                .trim()
                .parse()
                .map_err(|e| LobsterParseError::FieldParse {
                    field: "order_id",
                    source: e,
                })?,
            size: cols[3]
                .trim()
                .parse()
                .map_err(|e| LobsterParseError::FieldParse {
                    field: "size",
                    source: e,
                })?,
            price: cols[4]
                .trim()
                .parse()
                .map_err(|e| LobsterParseError::FieldParse {
                    field: "price",
                    source: e,
                })?,
            direction: parse_direction(cols[5].trim())?,
        })
    }

    /// Convert this row to an [`OrderCommand`] for the matching engine.
    ///
    /// Only order-entry events (new order, full cancel) produce commands.
    /// Execution reports, partial cancels, cross trades, and halts return `None`
    /// because those are *outputs* of matching, not inputs.
    pub fn to_command(&self) -> Option<OrderCommand> {
        match self.event_type {
            LobsterEventType::NewOrder => Some(OrderCommand::NewOrder {
                order_id: self.order_id,
                side: self.direction,
                price: self.price,
                qty: self.size,
            }),
            LobsterEventType::FullDelete => Some(OrderCommand::CancelOrder {
                order_id: self.order_id,
            }),
            _ => None,
        }
    }
}

/// LOBSTER CSV parser.
///
/// Parses LOBSTER message files (CSV, 6 columns) into [`LobsterRow`] structs
/// and converts applicable rows into [`OrderCommand`]s for the matching engine.
///
/// LOBSTER format: `Time,EventType,OrderID,Size,Price,Direction`
///
/// - **Time**: seconds after midnight (nanosecond precision)
/// - **EventType**: 1–7 (see [`LobsterEventType`])
/// - **Price**: USD × 10 000 (i.e. $91.14 → 911 400)
/// - **Direction**: 1 = buy, −1 = sell
///
/// Reference: <https://lobsterdata.com/info/DataStructure.php>
pub struct LobsterParser;

impl LobsterParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse all non-empty lines from a LOBSTER message CSV string.
    pub fn parse_messages(&self, content: &str) -> Result<Vec<LobsterRow>, LobsterParseError> {
        content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(LobsterRow::parse)
            .collect()
    }

    /// Extract order-entry commands from parsed LOBSTER rows, discarding
    /// execution reports and other non-entry events.
    pub fn extract_commands(&self, rows: &[LobsterRow]) -> Vec<OrderCommand> {
        rows.iter().filter_map(LobsterRow::to_command).collect()
    }
}

impl Default for LobsterParser {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_timestamp(s: &str) -> Result<u64, LobsterParseError> {
    let (secs_str, frac_str) = s.split_once('.').unwrap_or((s, ""));
    let secs: u64 = secs_str
        .parse()
        .map_err(|_| LobsterParseError::TimestampParse(s.to_owned()))?;
    if frac_str.is_empty() {
        return Ok(secs * 1_000_000_000);
    }
    let frac_len = frac_str.len().min(9);
    let frac_val: u64 = frac_str[..frac_len]
        .parse()
        .map_err(|_| LobsterParseError::TimestampParse(s.to_owned()))?;
    let nanos = frac_val * 10u64.pow((9 - frac_len) as u32);
    Ok(secs * 1_000_000_000 + nanos)
}

fn parse_event_type(s: &str) -> Result<LobsterEventType, LobsterParseError> {
    match s {
        "1" => Ok(LobsterEventType::NewOrder),
        "2" => Ok(LobsterEventType::PartialCancel),
        "3" => Ok(LobsterEventType::FullDelete),
        "4" => Ok(LobsterEventType::VisibleExecution),
        "5" => Ok(LobsterEventType::HiddenExecution),
        "6" => Ok(LobsterEventType::CrossTrade),
        "7" => Ok(LobsterEventType::TradingHalt),
        other => Err(LobsterParseError::InvalidEventType(other.to_owned())),
    }
}

fn parse_direction(s: &str) -> Result<Side, LobsterParseError> {
    match s {
        "1" => Ok(Side::Buy),
        "-1" => Ok(Side::Sell),
        other => Err(LobsterParseError::InvalidDirection(other.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> LobsterParser {
        LobsterParser::new()
    }

    // --- single-row parsing ---

    #[test]
    fn parse_new_order_buy() {
        let row = LobsterRow::parse("34200.018963654,1,4951489,100,2239800,1").unwrap();
        assert_eq!(row.timestamp_ns, 34_200_018_963_654);
        assert_eq!(row.event_type, LobsterEventType::NewOrder);
        assert_eq!(row.order_id, 4951489);
        assert_eq!(row.size, 100);
        assert_eq!(row.price, 2239800);
        assert_eq!(row.direction, Side::Buy);
    }

    #[test]
    fn parse_new_order_sell() {
        let row = LobsterRow::parse("34200.018963654,1,4951490,100,2239100,-1").unwrap();
        assert_eq!(row.direction, Side::Sell);
    }

    #[test]
    fn parse_full_delete() {
        let row = LobsterRow::parse("34200.019012357,3,4951489,100,2239800,1").unwrap();
        assert_eq!(row.event_type, LobsterEventType::FullDelete);
    }

    #[test]
    fn parse_visible_execution() {
        let row = LobsterRow::parse("34200.019012357,4,4951489,100,2239800,1").unwrap();
        assert_eq!(row.event_type, LobsterEventType::VisibleExecution);
    }

    // --- all event types ---

    #[test]
    fn parse_all_event_types() {
        for (code, expected) in [
            ("1", LobsterEventType::NewOrder),
            ("2", LobsterEventType::PartialCancel),
            ("3", LobsterEventType::FullDelete),
            ("4", LobsterEventType::VisibleExecution),
            ("5", LobsterEventType::HiddenExecution),
            ("6", LobsterEventType::CrossTrade),
            ("7", LobsterEventType::TradingHalt),
        ] {
            let line = format!("34200.0,{code},1,100,2239800,1");
            let row = LobsterRow::parse(&line).unwrap();
            assert_eq!(row.event_type, expected, "event type {code}");
        }
    }

    // --- timestamp precision ---

    #[test]
    fn timestamp_whole_seconds() {
        let row = LobsterRow::parse("34200,1,1,100,2239800,1").unwrap();
        assert_eq!(row.timestamp_ns, 34_200_000_000_000);
    }

    #[test]
    fn timestamp_millisecond_precision() {
        let row = LobsterRow::parse("34200.143,1,1,100,2239800,1").unwrap();
        assert_eq!(row.timestamp_ns, 34_200_143_000_000);
    }

    #[test]
    fn timestamp_microsecond_precision() {
        let row = LobsterRow::parse("34200.143426,1,1,100,2239800,1").unwrap();
        assert_eq!(row.timestamp_ns, 34_200_143_426_000);
    }

    #[test]
    fn timestamp_nanosecond_precision() {
        let row = LobsterRow::parse("34200.143426544,1,1,100,2239800,1").unwrap();
        assert_eq!(row.timestamp_ns, 34_200_143_426_544);
    }

    // --- error cases ---

    #[test]
    fn reject_invalid_event_type() {
        let result = LobsterRow::parse("34200.0,8,1,100,2239800,1");
        assert!(matches!(
            result,
            Err(LobsterParseError::InvalidEventType(_))
        ));
    }

    #[test]
    fn reject_invalid_direction() {
        let result = LobsterRow::parse("34200.0,1,1,100,2239800,2");
        assert!(matches!(
            result,
            Err(LobsterParseError::InvalidDirection(_))
        ));
    }

    #[test]
    fn reject_too_few_columns() {
        let result = LobsterRow::parse("34200.0,1,1,100,2239800");
        assert!(matches!(
            result,
            Err(LobsterParseError::WrongColumnCount(5))
        ));
    }

    #[test]
    fn reject_too_many_columns() {
        let result = LobsterRow::parse("34200.0,1,1,100,2239800,1,extra");
        assert!(matches!(
            result,
            Err(LobsterParseError::WrongColumnCount(7))
        ));
    }

    #[test]
    fn reject_non_numeric_order_id() {
        let result = LobsterRow::parse("34200.0,1,abc,100,2239800,1");
        assert!(matches!(result, Err(LobsterParseError::FieldParse { .. })));
    }

    // --- to_command conversion ---

    #[test]
    fn to_command_new_order() {
        let row = LobsterRow::parse("34200.0,1,42,100,5705000,1").unwrap();
        assert_eq!(
            row.to_command(),
            Some(OrderCommand::NewOrder {
                order_id: 42,
                side: Side::Buy,
                price: 5705000,
                qty: 100,
            })
        );
    }

    #[test]
    fn to_command_cancel_order() {
        let row = LobsterRow::parse("34200.0,3,42,100,5705000,1").unwrap();
        assert_eq!(
            row.to_command(),
            Some(OrderCommand::CancelOrder { order_id: 42 })
        );
    }

    #[test]
    fn to_command_execution_returns_none() {
        let row = LobsterRow::parse("34200.0,4,42,100,5705000,1").unwrap();
        assert_eq!(row.to_command(), None);
    }

    #[test]
    fn to_command_partial_cancel_returns_none() {
        let row = LobsterRow::parse("34200.0,2,42,50,5705000,1").unwrap();
        assert_eq!(row.to_command(), None);
    }

    #[test]
    fn to_command_hidden_execution_returns_none() {
        let row = LobsterRow::parse("34200.0,5,42,100,5705000,1").unwrap();
        assert_eq!(row.to_command(), None);
    }

    #[test]
    fn to_command_cross_trade_returns_none() {
        let row = LobsterRow::parse("34200.0,6,42,100,5705000,1").unwrap();
        assert_eq!(row.to_command(), None);
    }

    #[test]
    fn to_command_trading_halt_returns_none() {
        let row = LobsterRow::parse("34200.0,7,0,0,0,1").unwrap();
        assert_eq!(row.to_command(), None);
    }

    // --- LobsterParser ---

    #[test]
    fn parse_messages_multi_line() {
        let csv = "\
34200.018963654,1,4951489,100,2239800,1
34200.018963654,1,4951490,100,2239100,-1
34200.019012357,4,4951489,100,2239800,1
";
        let rows = parser().parse_messages(csv).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].event_type, LobsterEventType::NewOrder);
        assert_eq!(rows[0].direction, Side::Buy);
        assert_eq!(rows[1].direction, Side::Sell);
        assert_eq!(rows[2].event_type, LobsterEventType::VisibleExecution);
    }

    #[test]
    fn parse_messages_skips_blank_lines() {
        let csv = "\
34200.0,1,1,100,2239800,1

34200.0,1,2,200,2239900,-1
";
        let rows = parser().parse_messages(csv).unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn extract_commands_filters_non_entry_events() {
        let csv = "\
34200.0,1,1,100,2239800,1
34200.0,4,1,100,2239800,1
34200.0,3,2,100,2239800,-1
34200.0,7,0,0,0,1
";
        let p = parser();
        let rows = p.parse_messages(csv).unwrap();
        let cmds = p.extract_commands(&rows);
        assert_eq!(cmds.len(), 2);
        assert!(matches!(
            cmds[0],
            OrderCommand::NewOrder {
                order_id: 1,
                ..
            }
        ));
        assert!(matches!(cmds[1], OrderCommand::CancelOrder { order_id: 2 }));
    }
}
