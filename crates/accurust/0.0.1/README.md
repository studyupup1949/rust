# accurust
A Redis-backed scheduler for Rust applications.
<hr>

<h3>Motivation</h3>

Using `cron` for scheduling is common practice in app development. In my current quest to build an automated trading system, the requirement for a powerful, reliable, scalable task scheduler became apparent quite quickly. There can be complex dependence structure among the scheduled tasks; for example, say I need to update a trading position every hour but I first need to get the latest price data to compute the size of the new position. This problem enjoys an easy solution when getting data and updating the position are run at the same frequency: just bundle them into the same `cron` job; however, we need a more elegant solution when, say, we want to get new price data more often.
<hr>

<h3>Inspiration</h3>

In the Ruby ecosystem, they have `resque` and the `resque-scheduler` extension, for execution and scheduling respectively. I think the way this software works is very elegant and would like to emulate some features if possible. In particular, the scheduler does none of the heavy lifting of execution: when a scheduled job needs to be executed, the scheduler simply queues a YAML file, for which the executor will see to when a thread becomes free.
<hr>

<h3>Goal Functionality</h3>

* **Easy to use:**
* **Easy to test:**
* **Persistence:**
* **Powerful functionality:**
<hr>

### Plan & To-Do List

Coming soon.
<hr>
