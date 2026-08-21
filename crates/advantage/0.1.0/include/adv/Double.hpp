#ifndef _ADV_DOUBLE_HPP
#define _ADV_DOUBLE_HPP

namespace adv
{

class Double
{
public:
	/// \brief Destructor
	~Double();

	/// \brief Default constructor.
	Double();
	/// \brief Construct from a primitive
	Double(double);
	/// \brief Deleted copy constructor.
	Double(const Double&);
	/// \brief Move constructor.
	Double(Double&&);

	/// \brief Deleted copy assignment.
	Double& operator=(const Double&);
	/// \brief Move assignment.
	Double& operator=(Double&&);

	// Overloaded arithmetic operators
	Double operator+(const Double&) const;
	Double operator-(const Double&) const;
	Double operator*(const Double&) const;
	Double operator/(const Double&) const;

private:
	struct Impl;
	Impl* m_impl;

	Double(void* impl);
	const void* get_impl() const;

	friend class Context;

	friend Double operator+(double, const Double&);
	friend Double operator-(double, const Double&);
	friend Double operator*(double, const Double&);
	friend Double operator/(double, const Double&);

	friend Double sin(const Double&);
	friend Double cos(const Double&);
	friend Double tan(const Double&);
	friend Double abs(const Double&);
	friend Double exp(const Double&);
	friend Double ln(const Double&);

	friend Double min(const Double&, const Double&);
	friend Double max(const Double&, const Double&);
};

Double operator+(double, const Double&);
Double operator-(double, const Double&);
Double operator*(double, const Double&);
Double operator/(double, const Double&);

Double sin(const Double&);
Double cos(const Double&);
Double tan(const Double&);
Double abs(const Double&);
Double exp(const Double&);
Double ln(const Double&);

double sin(double);
double cos(double);
double tan(double);
double abs(double);
double exp(double);
double ln(double);

Double min(const Double&, const Double&);
Double max(const Double&, const Double&);

double min(double, double);
double max(double, double);

} // namespace adv

#endif // _ADV_DOUBLE_HPP

