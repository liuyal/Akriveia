# Design_Specification
Project to keep track of tasks and issues relating to Design Specification document

See grading rubric for document details

Due July 07, 2019

## To generate Glossary
pdflatex _Design_Specification.tex

makeindex -s _Design_Specification.ist -o _Design_Specification.gls _Design_Specification.glo

pdflatex _Design_Specification.tex

## To generate Reference
pdflatex _Design_Specification.tex

bibtex _Design_Specification.aux

pdflatex _Design_Specification.tex

pdflatex _Design_Specification.tex