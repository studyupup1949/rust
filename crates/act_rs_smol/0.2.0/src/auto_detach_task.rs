//use std::ops::Deref;

use smol::Task;

///
/// Calls the detach method of the task when dropped.
/// 
pub struct AutoDetachTask<T, M = ()>
{

    task: Option<Task<T, M>>

}

impl<T, M> AutoDetachTask<T, M>
{

    pub fn new(task: Task<T, M>) -> Self
    {

        Self
        {

            task: Some(task)

        }

    }

    pub fn task_ref(&self) -> &Task<T, M>
    {

        self.task.as_ref().expect("Task must be present.")

    } 

    pub fn take(mut self) -> Task<T, M>
    {

        self.task.take().expect("Task must be present.")

    }

}

//Disbaled

/*
impl<T, M> Deref for AutoDetachTask<T, M>
{

    type Target = Task<T, M>;

    fn deref(&self) -> &Self::Target
    {

        self.task.as_ref().expect("Task must be present.")

    }

}
*/

impl<T, M> Drop for AutoDetachTask<T, M>
{

    fn drop(&mut self)
    {

        if let Some(task) = self.task.take()
        {

            task.detach();

        }

    }

}
