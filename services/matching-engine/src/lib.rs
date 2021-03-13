use matchbook_types::*;
use std::cmp::Reverse;
use std::{
    cmp::PartialOrd,
    collections::{BinaryHeap, HashMap},
};

#[derive(Debug, Default)]
pub struct MatchingEngine {
    books: HashMap<SymbolOwned, Book>,
}

impl MatchingEngine {
    pub fn submit_limit_order(
        &mut self,
        side: Side,
        symbol: SymbolRef,
        price: Price,
        quantity: Quantity,
    ) -> Result<Vec<Execution>, Box<dyn std::error::Error>> {
        let order = LimitOrder::new(quantity, price);

        let book = match self.books.get_mut(symbol) {
            Some(book) => book,
            None => return Err(format!("symbol '{:?}' does not exist", symbol).into()),
        };

        let fills = match side {
            Side::Ask => book.submit_limit_ask(order)?,
            Side::Bid => book.submit_limit_bid(order)?,
        };

        Ok(fills)
    }

    /// Create a symbol if it doesn't exist and return true, otherwise do nothing and return false
    pub fn create_symbol(&mut self, symbol: SymbolOwned) -> bool {
        self.books.insert(symbol, Book::default()).is_none()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Book {
    bids: BinaryHeap<Reverse<LimitOrder>>,
    asks: BinaryHeap<LimitOrder>,
}

impl Book {
    /// Submit a ask limit order
    ///
    /// # Examples
    ///
    /// ```
    /// use matching_engine::{Book, LimitOrder};
    /// let mut book = Book::default();
    /// let first_order = LimitOrder::new(100, 10);
    /// let _ = book.submit_limit_ask(first_order).unwrap();
    ///
    /// let second_order = LimitOrder::new(150, 10);
    /// let _ = book.submit_limit_ask(second_order);
    ///
    /// assert_eq!(book.asks().nth(0), Some(&first_order));
    /// assert_eq!(book.asks().nth(1), Some(&second_order));
    ///
    /// let better_order = LimitOrder::new(150, 11);
    /// let _ = book.submit_limit_ask(better_order);
    /// assert_eq!(book.asks().nth(0), Some(&better_order));
    /// ```
    pub fn submit_limit_ask(
        &mut self,
        order: LimitOrder,
    ) -> Result<Vec<Execution>, Box<dyn std::error::Error>> {
        self.asks.push(order);
        Ok(self.fill_matching())
    }

    /// Submit a bid limit order
    ///
    /// # Examples
    ///
    /// ```
    /// use matching_engine::{Book, LimitOrder};
    /// let mut book = Book::default();
    /// let first_order = LimitOrder::new(100, 10);
    /// let _ = book.submit_limit_bid(first_order).unwrap();
    ///
    /// let second_order = LimitOrder::new(150, 10);
    /// let _ = book.submit_limit_bid(second_order);
    ///
    /// assert_eq!(book.bids().nth(0), Some(&first_order));
    /// assert_eq!(book.bids().nth(1), Some(&second_order));
    ///
    /// let better_order = LimitOrder::new(150, 9);
    /// let _ = book.submit_limit_bid(better_order);
    /// assert_eq!(book.bids().nth(0), Some(&better_order));
    /// ```
    pub fn submit_limit_bid(
        &mut self,
        order: LimitOrder,
    ) -> Result<Vec<Execution>, Box<dyn std::error::Error>> {
        self.bids.push(Reverse(order));
        Ok(self.fill_matching())
    }

    /// Attempt to fill any orders
    fn fill_matching(&mut self) -> Vec<Execution> {
        let mut fills = vec![];

        'matching: loop {
            if let Some(mut bid) = self.bids.peek_mut() {
                let mut ask = match self.asks.peek_mut() {
                    Some(ask) if ask.price >= bid.0.price => ask,
                    _ => break 'matching,
                };
                let fillable_quantity = ask.remaining().min(bid.0.remaining());

                ask.fill(fillable_quantity);
                bid.0.fill(fillable_quantity);

                fills.push(Execution {
                    price: ask.price.max(bid.0.price),
                    quantity: fillable_quantity,
                });
            } else {
                break 'matching;
            }

            match self.bids.peek().clone() {
                Some(Reverse(bid)) if bid.is_filled() => {
                    self.bids.pop();
                }
                _ => {}
            };

            match self.asks.peek().clone() {
                Some(ask) if ask.is_filled() => {
                    self.asks.pop();
                }
                _ => {}
            };
        }

        fills
    }

    pub fn asks(&self) -> impl Iterator<Item = &LimitOrder> {
        self.asks.iter()
    }

    pub fn bids(&self) -> impl Iterator<Item = &LimitOrder> {
        self.bids.iter().map(|Reverse(x)| x)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Execution {
    pub quantity: Quantity,
    pub price: Price,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct LimitOrder {
    quantity: Quantity,
    price: Price,
    remaining: Quantity,
}

impl PartialOrd for LimitOrder {
    fn partial_cmp(&self, other: &LimitOrder) -> Option<std::cmp::Ordering> {
        self.price.partial_cmp(&other.price)
    }
}

impl std::cmp::Ord for LimitOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.price.cmp(&other.price)
    }
}

impl LimitOrder {
    pub fn new(quantity: usize, price: usize) -> Self {
        LimitOrder {
            quantity,
            price,
            remaining: quantity,
        }
    }

    pub fn price(&self) -> Price {
        self.price
    }

    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    pub fn remaining(&self) -> Quantity {
        self.remaining
    }

    /// Fill some shares, returning the shares that still need to be filled
    pub fn fill(&mut self, quantity: Quantity) -> Quantity {
        assert!(self.remaining >= quantity);
        self.remaining -= quantity;
        self.remaining
    }

    pub fn is_filled(&self) -> bool {
        self.remaining == 0
    }
}
