#include <gtest/gtest.h>
#include <adv.hpp>

TEST(Double, arithmetic_ops)
{
	adv::Context ctx;
	auto a = ctx.new_independent();
	auto b = ctx.new_independent();

	a + b;
	a - b;
	a * b;
	a / b;

	a + 1.0;
	a - 1.0;
	a * 1.0;
	a / 1.0;

	1.0 + a;
	1.0 - a;
	1.0 * a;
	1.0 / a;
}

TEST(Double, unary_functions)
{
	adv::Context ctx;
	auto x = ctx.new_independent();

	adv::sin(x);
	adv::cos(x);
	adv::tan(x);
	adv::abs(x);
	adv::exp(x);
	adv::ln(x);
}

TEST(Double, binary_functions)
{
	adv::Context ctx;
	auto a = ctx.new_independent();
	auto b = ctx.new_independent();

	adv::min(a, b);
	adv::min(a, 1.0);
	adv::min(1.0, a);
	ASSERT_EQ(0.0, adv::min(1.0, 0.0));

	adv::max(a, b);
	adv::max(a, 1.0);
	adv::max(1.0, a);
	adv::max(1.0, 0.0);
	ASSERT_EQ(1.0, adv::max(1.0, 0.0));
}
