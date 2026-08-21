# AbleCI-Registry
Registries for AbleCI Class and Vendor IDs

This registry is jointly run by the Clever-ISA project and AbleCorp

## How to file for a class id

Class IDs are assigned on a first come first serve basis.
The Clever-ISA project assigns class ids from 0x8000 through 0xFF7F. 

Additionally, Class IDs from 0x0000-0x007F and 0xFF80-0xFFFF are managed jointly and are considered "well-known" and apply to well-known device classes - check if one exists. 

To file a request for a class id with this repository, submit a pull request adding to class-id-registry.csv, a row with the following information:
* The class id requested (to make review quicker, place it in sorted order based on class id), which must not be already in use and must be within the assignment range.
* The internal name of the class, which is consists of lowercase ascii letters, underscores, and dashes.
* The public name of the class,
* The name and contact information (email address) for the person or organization who owns and maintains the subclass registry for the class id. Each class registrant must maintain a registry for subclasses and rules for subclass registration within that registry.,
* The name and contact information (email address) for the person filing the registration and who is responsible for the registration,
* The name and contact information of the registrar, Clever-ISA Project <cleverisa@lcdev.xyz>.

Class ID filings are reviewed according to standard Clever-ISA Governance.

## How to file for a vendor ID

Vendor IDs are assigned on a first come first serve basis.
The Clever-ISA project assigns vendor ids from 0x8000 through 0xFFFE. 

To file a request for a vendor id with this repository, submit a pull request adding to vendor-id-registry.csv, a row with the following information:
* The vendor id requested (to make review quicker, place it in sorted order based on vendor id), which must not be already in use and must be within the assignment range.
* The internal name of the class, which is consists of lowercase ascii letters, underscores, and dashes.
* The public name of the class,
* The name and contact information (email address) for the person or organization who owns and maintains the product id registry for the vendor id. Each vendor registrant must maintain a list of product ids issued under the vendor id,
* The name and contact information (email address) for the person filing the registration and who is responsible for the registration,
* The name and contact information of the registrar, Clever-ISA Project <cleverisa@lcdev.xyz>.

Vendor ID filings are reviewed according to standard Clever-ISA Governance.

## How to file for a subclass id of a Well-Known Class

Subclass IDs for Well-Known classes are issued by agreement of both the Clever-ISA Project and AbleCorp. Such Subclass IDs are issued when a specification exists for the subclass protocol, and the subclass matches the standard behaviour of expected of class. 

To file a request for a vendor id with this repository, submit a pull request adding to the appropriate file in wellknown-subclasses-register, a row with the following information:
* The subclass id requested (to make review quicker, place it in sorted order based on vendor id), which must not be already in use and must be within the assignment range.
* The internal name of the class, which is consists of lowercase ascii letters, underscores, and dashes.
* The public name of the class,
* A link to the specification for the subclass,
* The name and contact information (email address) for the person filing the registration and who is responsible for the registration,
* The name and contact information of the registrar, Clever-ISA Project <cleverisa@lcdev.xyz>.

Registrations of Well-known subclasses are approved according to the standard Clever-ISA Governance. 

## Amending Registration Info

The registrant of a class id, vendor id, or well-known subclass id may amend the information given in the registration.
The registrant may also declare a list of delegates who are authorized to amend that information.

Amending Registration Info requires Maintainer Approval only, except that amending specification information for a well-known subclass id registration requires full approval.

