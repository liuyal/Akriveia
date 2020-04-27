# Requirements Specification

Project to keep track of tasks and issues relating to Requirements Specification document

See grading rubric for document details

Due June 09, 2019



## To generate Glossary

pdflatex _Requirements_Specification.tex

makeindex -s _Requirements_Specification.ist -o _Requirements_Specification.gls _Requirements_Specification.glo

pdflatex _Requirements_Specification.tex



## To generate Reference

pdflatex _Requirements_Specification.tex

bibtex _Requirements_Specification.aux

pdflatex _Requirements_Specification.tex

pdflatex _Requirements_Specification.tex