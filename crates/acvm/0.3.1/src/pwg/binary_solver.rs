// There are cases when knowing that a witness can be more than one value
// can lead to us solving it. ie solving a system of simultaneous equations.
//
// For example, consider the two following equations;
// (1) xy - y = 0
// (2) x = 0
//
// The first equation tells us that y = 0 or x = 1 for the equation to be satisfied.
// The second equation tells us that x = 0, which means that the first equation must have y = 0
//
// The binary solver solves these simultaneous equations in the case where
// we can deduce that a variable is either 0 or 1; binary.
