# acetylene-parser

A string parser for different chemical nomenclature.

## functions

- `tokenize(string, type="formula") -> Substance`

  Tokenizes a string describing a chemical, yielding a Substance with (optional)
  functional groups corresponding to (more) fundamental components.

  - "formula" type expects a simple "secondary school" element-symbol naming
  string.

  - "smiles" expects a chemical name utilizing the
    [SMILES system](https://en.wikipedia.org/wiki/Simplified_molecular-input_line-entry_system).
  - **TODO**: "iupac" expects a chemical name utilizing the
    [IUPAC system](https://en.wikipedia.org/wiki/IUPAC_nomenclature_of_organic_chemistry).

## roadmap

- [x] implement SMILES parsing
- [ ] improve struct based on SMILES findings
- [ ] decide whether InChI implementation is worth it

## references

### SMILES

- https://en.wikipedia.org/wiki/Simplified_molecular-input_line-entry_system
- http://opensmiles.org/opensmiles.html
- http://www.dalkescientific.com/writings/diary/archive/2004/01/05/tokens.html

### IUPAC

- http://www.chem.uiuc.edu/GenChemReferences/nomenclature_rules.html
- https://web.archive.org/web/20100626004648/http://www.acdlabs.co.uk/iupac/nomenclature/93/r93_125.htm
- https://en.wikipedia.org/wiki/IUPAC_nomenclature_of_organic_chemistry
- https://bitbucket.org/dan2097/opsin/src

### InChI

- http://iupac.org/who-we-are/divisions/division-details/inchi/
- http://www.inchi-trust.org/downloads/

### data

- http://www.chemicalize.org/blog/
- http://www.chemspider.com/
