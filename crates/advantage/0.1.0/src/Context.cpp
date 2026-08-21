#include <adv/Context.hpp>
#include "ffi.hpp"

#include <stdexcept>

namespace adv
{

struct Context::Impl
{
public:
	adv_context* ctx;
};

Context::~Context()
{
	if (m_impl != nullptr) {
		if (m_impl->ctx != nullptr) {
			::adv_context_free(m_impl->ctx);
		}
		delete m_impl;
	}
}

Context::Context():
	m_impl(new Impl)
{
	m_impl->ctx = ::adv_context_new();
}

Context::Context(Context&& other):
	m_impl(other.m_impl)
{
	other.m_impl = nullptr;
}

Context& Context::operator=(Context&& other)
{
	if (m_impl != nullptr) {
		::adv_context_free(m_impl->ctx);
		delete m_impl;
	}
	m_impl = other.m_impl;
	other.m_impl = nullptr;
	return *this;
}

void* Context::get_impl()
{
	return m_impl->ctx;
}

void* Context::move_impl() {
	auto ctx = m_impl->ctx;
	delete m_impl;
	m_impl = nullptr;
	return ctx;
}

Double Context::new_independent()
{
	return Double(::adv_context_new_independent(m_impl->ctx));
}

void Context::set_dependent(const Double& var)
{
	::adv_context_set_dependent(m_impl->ctx, reinterpret_cast<const ::adv_double*>(var.get_impl()));
}

} // namespace adv
