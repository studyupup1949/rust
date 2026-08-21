// This file is used by build.rs to verify the underlying type of an enum, when
// left implicit, is the expected type given a minimum and maximum value.

#include <type_traits>

enum test_enum {
    min = TEST_ENUM_MIN,
    max = TEST_ENUM_MAX,
};

static_assert(
    std::is_same<
        TEST_ENUM_TYPE,
        std::underlying_type<test_enum>::type
    >::value,
    "test_enum didn't match expected type"
);
