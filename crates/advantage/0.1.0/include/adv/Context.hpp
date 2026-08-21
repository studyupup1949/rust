#ifndef _ADV_CONTEXT_HPP
#define _ADV_CONTEXT_HPP

#include "Double.hpp"

namespace adv
{

class Context
{
public:
	/// \brief Destructor
	~Context();

	/// \brief Default constructor.
	Context();
	/// \brief Deleted copy constructor.
	Context(const Context&) = delete;
	/// \brief Move constructor.
	Context(Context&&);

	/// \brief Deleted copy assignment.
	Context& operator=(const Context&) = delete;
	/// \brief Move assignment.
	Context& operator=(Context&&);

	/// Get a new independent variable
	Double new_independent();
	/// Set a variable dependent
	void set_dependent(const Double& var);

private:
	struct Impl;
	Impl* m_impl;

	friend class Tape;
	void* get_impl();
	void* move_impl();
};

} // namespace adv

#endif // _ADV_CONTEXT_HPP
