macro(adv_test name)
	add_executable(${name} ${name}.cpp)
	target_link_libraries(${name}
		advantage
		GTest::GTest
		GTest::Main
	)
	gtest_discover_tests(${name})
endmacro()

adv_test(ContextTests)
adv_test(DoubleTests)
