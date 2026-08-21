#include <adv/Double.hpp>
#include "ffi.hpp"

#include <cmath>
#include <stdexcept>

namespace adv
{

struct Double::Impl
{
public:
	adv_double* val;
};

Double::~Double()
{
	::adv_double_free(m_impl->val);
	delete m_impl;
}

Double::Double(void* impl):
	m_impl(new Impl)
{
	m_impl->val = reinterpret_cast<adv_double*>(impl);
}

Double::Double():
	Double(::adv_double_default())
{
}

Double::Double(double val):
	Double(::adv_double_from_value(val))
{
}

Double::Double(const Double& other):
	Double(::adv_double_copy(other.m_impl->val))
{
}

Double::Double(Double&& other):
	Double(other.m_impl->val)
{
	other.m_impl->val = ::adv_double_default();
}

Double& Double::operator=(Double&& other)
{
	::adv_double_free(m_impl->val);
	m_impl->val = other.m_impl->val;
	other.m_impl->val = ::adv_double_default();
	return *this;
}

Double& Double::operator=(const Double& other)
{
	::adv_double_free(m_impl->val);
	m_impl->val = ::adv_double_copy(other.m_impl->val);
	return *this;
}

const void* Double::get_impl() const
{
	return m_impl->val;
}

#define BINARY_OP_IMPL(NAME, OP) \
	Double Double::operator OP(const Double& rhs) const \
	{ \
		::adv_double* result_impl; \
		::adv_op_##NAME(m_impl->val, rhs.m_impl->val, &result_impl); \
		return Double(result_impl); \
	} \
	\
	Double operator OP(double lhs_, const Double& rhs) \
	{ \
		auto lhs = Double(lhs_); \
		return lhs + rhs; \
	}
BINARY_OP_IMPL(add, +)
BINARY_OP_IMPL(sub, -)
BINARY_OP_IMPL(mul, *)
BINARY_OP_IMPL(div, /)
#undef BINARY_OP_IMPL

#define UNARY_FUNC_IMPL(NAME, STDNAME) \
	Double NAME(const Double& val) { \
		::adv_double* ptr; \
		::adv_##NAME(val.m_impl->val, &ptr); \
		return Double(ptr); \
	} \
	\
	double NAME(double val) { \
		return std::STDNAME(val); \
	}

UNARY_FUNC_IMPL(sin, sin)
UNARY_FUNC_IMPL(cos, cos)
UNARY_FUNC_IMPL(tan, tan)
UNARY_FUNC_IMPL(abs, abs)
UNARY_FUNC_IMPL(exp, exp)
UNARY_FUNC_IMPL(ln, log)
#undef UNARY_FUNC_IMPL

#define BINARY_FUNC_IMPL(NAME) \
	Double NAME(const Double& lhs, const Double& rhs) { \
		::adv_double* ptr; \
		::adv_##NAME(lhs.m_impl->val, rhs.m_impl->val, &ptr); \
		return Double(ptr); \
	}
BINARY_FUNC_IMPL(min)
BINARY_FUNC_IMPL(max)
#undef BINARY_FUNC_IMPL

double min(double lhs, double rhs) {
	if (lhs < rhs) {
		return lhs;
	}
	else {
		return rhs;
	}
}

double max(double lhs, double rhs) {
	if (lhs < rhs) {
		return rhs;
	}
	else {
		return lhs;
	}
}

} // namespace adv

