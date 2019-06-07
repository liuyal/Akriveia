# Design_Specification
Project to keep track of tasks and issues relating to Design Specification document

See grading rubric for document details

Due July 07, 2019

## To generate Glossary
pdflatex _Deisgn_Specification.tex

makeindex -s _Deisgn_Specification.ist -o _Deisgn_Specification.gls _Deisgn_Specification.glo

pdflatex _Deisgn_Specification.tex

## To generate Reference
pdflatex _Deisgn_Specification.tex

bibtex _Deisgn_Specification

pdflatex _Deisgn_Specification.tex

pdflatex _Deisgn_Specification.tex