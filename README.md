# Design_Specification
Project to keep track of tasks and issues relating to Design Specification document

See grading rubric for document details

Due July 07, 2019

## To generate Glossary
pdflatex __Deisgn_Specification.tex

makeindex -s __Deisgn_Specification.ist -o __Deisgn_Specification.gls __Deisgn_Specification.glo

pdflatex __Deisgn_Specification.tex

## To generate Reference
pdflatex __Deisgn_Specification.tex

bibtex __Deisgn_Specification

pdflatex __Deisgn_Specification.tex

pdflatex __Deisgn_Specification.tex