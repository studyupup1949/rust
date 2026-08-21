# accrua-rs
accrua-rs aims to be an accruals calculation library, focusing on the OTC derivatives market
and providing users the tools as defined in the ISDA.

This is a pre-alpha version, which, as of now, I consider a placeholder, rather than a library that
could be used in any environment.

### TODO
- [X] Business calendar trait (providing methods for the date adjustment)
- [ ] The most commonly used business calendars
- [ ] Floating rate index type/trait
- [ ] Money/currency type (will use an external crate most likely)
- [ ] Day count fractions and fuctions used for their calculation
- [ ] Compounding
- [ ] Currency holiday calendar
- [ ] Fixed income types
    - [ ] Bonds
    - [ ] Swaps/Swap legs