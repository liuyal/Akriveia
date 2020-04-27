# Document_Project_Proposal

Project to keep track of tasks and issues relating to Project Proposal document

See grading rubric for document details

Due July 21, 2019

## To generate Glossary
pdflatex _Project_Proposal.tex

makeindex -s _Project_Proposal -o _Project_Proposal.gls _Project_Proposal.glo

pdflatex _Project_Proposal.tex

## To generate Reference
pdflatex _Project_Proposal.tex

bibtex _Project_Proposal

pdflatex _Project_Proposal.tex

pdflatex _Project_Proposal.tex