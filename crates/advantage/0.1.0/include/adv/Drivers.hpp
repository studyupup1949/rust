#ifndef _ADV_DRIVERS_HPP
#define _ADV_DRIVERS_HPP

#include "Tape.hpp"

#include <vector>

namespace adv
{

std::vector<double> zero_order(const Tape& tape, const std::vector<double>& x);
std::pair<std::vector<double>, std::vector<double>> first_order(const Tape& tape, const std::vector<double>& x, const std::vector<double>& dx);
std::pair<std::vector<double>, std::vector<double>> first_order_reverse(const Tape& tape, const std::vector<double>& x, const std::vector<double>& ybar);

std::vector<double> jacobian(const Tape& tape, const std::vector<double>& x);
std::vector<double> jacobian_reverse(const Tape& tape, const std::vector<double>& x);

struct AbsNormalForm {
	std::size_t n;
	std::size_t m;
	std::size_t s;

	std::vector<double> a;
	std::vector<double> Z;
	std::vector<double> L;

	std::vector<double> b;
	std::vector<double> J;
	std::vector<double> Y;
};

AbsNormalForm abs_normal(const Tape& tape, const std::vector<double>& x);

} // namespace adv

#endif // _ADV_DRIVERS_HPP
