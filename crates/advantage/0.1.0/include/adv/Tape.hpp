#ifndef _ADV_TAPE_HPP
#define _ADV_TAPE_HPP

#include "Context.hpp"

#include <vector>

namespace adv
{

// Forward decls
struct AbsNormalForm;

/// \brief Sequence of recorded operation representing an arithmetic graph
class Tape final
{
public:
	/// \brief Destructor.
	~Tape();

	/// \brief Default constructor.
	Tape() = delete;
	/// \brief Copy constructor.
	Tape(const Tape&) = delete;
	/// \brief Move constructor.
	Tape(Tape&&);
	/// \brief Construct a tape from a context
	Tape(Context&& ctx);

	/// \brief Copy assignment.
	Tape& operator=(const Tape&) = delete;
	/// \brief Move assignment.
	Tape& operator=(Tape&&);

	/// \brief Number of independent vars.
	std::size_t num_indeps() const;
	/// \brief Number of dependent vars.
	std::size_t num_deps() const;
	/// \brief Number of abs evaluations.
	std::size_t num_abs() const;

	/// Decompose an abs-factorable function
	Tape abs_decompose() const;

private:
	struct Impl;
	Impl* m_impl;

	Tape(void*);

	const void* get_impl() const;

	friend std::vector<double> zero_order(const Tape&, const std::vector<double>&);
	friend std::pair<std::vector<double>, std::vector<double>> first_order(const Tape& tape, const std::vector<double>& x, const std::vector<double>& dx);
	friend std::pair<std::vector<double>, std::vector<double>> first_order_reverse(const Tape& tape, const std::vector<double>& x_, const std::vector<double>& ybar_);
	friend std::vector<double> jacobian(const Tape& tape, const std::vector<double>& x);
	friend std::vector<double> jacobian_reverse(const Tape& tape, const std::vector<double>& x);
	friend AbsNormalForm abs_normal(const Tape& tape, const std::vector<double>& x);
};

}

#endif // _ADV_TAPE_HPP
