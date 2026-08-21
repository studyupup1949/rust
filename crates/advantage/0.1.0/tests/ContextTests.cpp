#include <gtest/gtest.h>
#include <adv.hpp>

TEST(Context, init)
{
	adv::Context ctx;
}

TEST(Context, get_doubles)
{
	adv::Context ctx;
	auto v1 = ctx.new_independent();
	ctx.set_dependent(v1);
}

TEST(Context, record_simple_tape)
{
	adv::Context ctx;
	auto v1 = ctx.new_independent();
	auto v2 = v1 * v1;
	ctx.set_dependent(v2);
}
