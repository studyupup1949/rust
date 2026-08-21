# Asset Administration Types

A crate for mapping the types of Asset Administration Shells to
Rust.

## Structure

For supporting multiple versions of the AAS specifications,  
each part of the Spec (Part 1, Part 3...) is going to be seperated,  
as well as individually versions.   
I.e. you can find the types of Part 1 version 3.1 under `src/part_1/v3_1`.

## Goals:

- Provide as much type-safety as possible
    - Parsing types to ensure proper formats.
    - Constraints are to be respected.
- Provide support for JSON and XML simultaneous
- Implement all AAS specs
- As many tests as possible
- Code-Documentation based on the AAS spec.
  Close to no need to consult spec papers.

For 1.0 JSON, XML and .AASX Format should work.
As many tests as possible. Test multiple demo Shells.

## Current State
As of now, following features are implemented (but not tested thoroughly)
- JSON de-/serialisation
  - OPENAPI Spec generation
  - Generic implementation of AAS Server API for Axum
- XML de-/serialisation

## To be done (contributions more than welcome!):
- More Testing. Every bit should be tested 
  - More tests of demo AAS.
- AASX File Format (AAS Spec 5)
- Cleaning up the code
  - For better usability
  - to support JSON and XML simultaneously
- RDF Format
- JSON Metadata/Value-Only
- XML Metadata/Value-Only
- Documentation. At best reference pages/anchors of official spec.
