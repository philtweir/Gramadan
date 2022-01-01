using System;
using System.Collections.Generic;
using Gramadan;
using System.IO;
using System.Xml;

namespace Tester
{
    class TestPossessives
    {
        public static void PossNP() {
            List<Noun> nouns=new List<Noun>();
            nouns.Add(new Noun(@"data/noun/árasán_masc1.xml"));
            nouns.Add(new Noun(@"data/noun/bó_fem.xml"));
            nouns.Add(new Noun(@"data/noun/comhlacht_masc3.xml"));
            nouns.Add(new Noun(@"data/noun/dealbh_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/éiceachóras_masc1.xml"));
            nouns.Add(new Noun(@"data/noun/francfurtar_masc1.xml"));
            nouns.Add(new Noun(@"data/noun/fliúit_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/fadhb_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/fobhríste_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/garáiste_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/haematóma_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/iasacht_fem3.xml"));
            nouns.Add(new Noun(@"data/noun/jab_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/leabharlann_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/máthair_fem5.xml"));
            nouns.Add(new Noun(@"data/noun/nóta_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/ócáid_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/pacáiste_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/rás_masc3.xml"));
            nouns.Add(new Noun(@"data/noun/sobaldráma_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/sábh_masc1.xml"));
            nouns.Add(new Noun(@"data/noun/stábla_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/sráid_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/tábhairne_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/ubh_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/x-gha_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/zombaí_masc4.xml"));

            List<Possessive> possessives=new List<Possessive>();
            possessives.Add(new Possessive(@"data/possessive/mo_poss.xml"));
            possessives.Add(new Possessive(@"data/possessive/do_poss.xml"));
            possessives.Add(new Possessive(@"data/possessive/a_poss_masc.xml"));
            possessives.Add(new Possessive(@"data/possessive/a_poss_fem.xml"));
            possessives.Add(new Possessive(@"data/possessive/ár_poss.xml"));
            possessives.Add(new Possessive(@"data/possessive/bhur_poss.xml"));
            possessives.Add(new Possessive(@"data/possessive/a_poss_pl.xml"));

            Adjective adj=new Adjective(@"data/adjective/mór_adj1.xml");

            StreamWriter writer=new StreamWriter(@"test-poss-np.txt");
            foreach(Noun n in nouns) {
                foreach(Possessive poss in possessives) {
                    NP np = new NP(n, adj, poss);
                    writer.WriteLine(poss.getFriendlyNickname()+"\t"+np.sgNom[0].value+"\t"+np.sgGen[0].value+"\t"+np.plNom[0].value+"\t"+np.plGen[0].value);
                    Console.WriteLine(np.print());
                }
                writer.WriteLine();
            }
            writer.Close();
        }

        public static void PrepPossNP() {
            List<Noun> nouns=new List<Noun>();
            nouns.Add(new Noun(@"data/noun/árasán_masc1.xml"));
            nouns.Add(new Noun(@"data/noun/bó_fem.xml"));
            nouns.Add(new Noun(@"data/noun/comhlacht_masc3.xml"));
            nouns.Add(new Noun(@"data/noun/dealbh_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/éiceachóras_masc1.xml"));
            nouns.Add(new Noun(@"data/noun/francfurtar_masc1.xml"));
            nouns.Add(new Noun(@"data/noun/fliúit_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/fadhb_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/fobhríste_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/garáiste_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/haematóma_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/iasacht_fem3.xml"));
            nouns.Add(new Noun(@"data/noun/jab_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/leabharlann_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/máthair_fem5.xml"));
            nouns.Add(new Noun(@"data/noun/nóta_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/ócáid_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/pacáiste_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/rás_masc3.xml"));
            nouns.Add(new Noun(@"data/noun/sobaldráma_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/sábh_masc1.xml"));
            nouns.Add(new Noun(@"data/noun/stábla_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/sráid_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/tábhairne_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/ubh_fem2.xml"));
            nouns.Add(new Noun(@"data/noun/x-gha_masc4.xml"));
            nouns.Add(new Noun(@"data/noun/zombaí_masc4.xml"));

            List<Possessive> possessives=new List<Possessive>();
            possessives.Add(new Possessive(@"data/possessive/mo_poss.xml"));
            possessives.Add(new Possessive(@"data/possessive/do_poss.xml"));
            possessives.Add(new Possessive(@"data/possessive/a_poss_masc.xml"));
            possessives.Add(new Possessive(@"data/possessive/a_poss_fem.xml"));
            possessives.Add(new Possessive(@"data/possessive/ár_poss.xml"));
            possessives.Add(new Possessive(@"data/possessive/bhur_poss.xml"));
            possessives.Add(new Possessive(@"data/possessive/a_poss_pl.xml"));

            Adjective adj=new Adjective(@"data/adjective/mór_adj1.xml");

            List<Preposition> prepositions=new List<Preposition>();
            prepositions.Add(new Preposition(@"data/preposition/ag_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/ar_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/as_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/chuig_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/de_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/do_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/faoi_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/i_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/le_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/ó_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/roimh_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/thar_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/trí_prep.xml"));
            prepositions.Add(new Preposition(@"data/preposition/um_prep.xml"));

            StreamWriter writer=new StreamWriter(@"test-prep-poss-np.txt");
            foreach(Preposition prep in prepositions) {
                foreach(Noun n in nouns) {
                    foreach(Possessive poss in possessives) {
                        NP np = new NP(n, adj, poss);
                        PP pp=new PP(prep, np);
                        writer.WriteLine("["+prep.getLemma()+" + "+poss.getFriendlyNickname()+"]\t"+pp.sg[0].value+"\t"+pp.pl[0].value);
                        Console.WriteLine(pp.print());
                    }
                    writer.WriteLine();
                }
            }
            writer.Close();
        }
    }
}
