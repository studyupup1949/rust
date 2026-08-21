Retail Point-of-Sale Sample
---------------------------

This directory contains a policy for a fictional retail point-of-sale
(POS) system that serves to demonstrate the ALFA syntax and give users
a known-working example that `a2x` can convert to XACML and inspect.

In this scenario, we are building an authorization system to control
point-of-sale systems in a retail environment.

Our POS system will have different types of users, primarily
identified by their role (customer, clerk, supervisor), and will use a
variety of attributes defined in the `attr.alfa` file.

Some of the scenarios implemented in ALFA:

* Voiding a transaction will require supervisor approval (RBAC).
* Prohibit customer actions outside of business hours (environment
  attributes).
* Require age verification for restricted items (obligation).
* Record clerk's assistance adding items to a customer cart (advice).
