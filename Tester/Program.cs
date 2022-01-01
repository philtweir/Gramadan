using System;
using System.Collections.Generic;
using Gramadan;
using Tester;
using System.IO;
using System.Xml;
using System.Xml.Linq;

namespace Tester
{
	class Program
	{
		static void Main(string[] args)
		{
			Consistency.Nouns();
			Consistency.VerbalNouns();
			Consistency.Adjectives();
			Consistency.Similarity();
			QualityControl.AinmfhocailBearnai();
			QualityControl.AidiachtaiBearnai();
			QualityControl.BriathraInfinideachaBearnai();
			QualityControl.BriathraFinideachaBearnai();
			// // QualityControl.AinmfhocailRegex();
			// // QualityControl.AidiachtaiRegex();
			// // Gadaíocht <-- no input available
			TestPossessives.PossNP();
			TestPossessives.PrepPossNP();
			ShortTest();
			FindAll();
			Go();
			Console.Write("Déanta."); Console.ReadLine();
		}

		//Just quickly test something:
		public static void ShortTest() {
			Noun n = new Noun(@"data/noun/Gael_masc1.xml");
			Adjective adj = new Adjective(@"data/adjective/Gaelach_adj1.xml");
			NP np = new NP(n, adj);
			Console.WriteLine(np.print());
		}

		//Find all words that have some property:
		public static void FindAll() {
			foreach(string file in Directory.GetFiles(@"data/noun")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				Noun noun=new Noun(doc);
				foreach(FormPlGen form in noun.plGen) {
					if(form.strength == Strength.Weak && Opers.IsSlender(form.value)) {
						Console.WriteLine(form.value);
					}
				}
			}
		}

		/// <summary>
		/// Resaves BuNaMo entries (for example to update their file names).
		/// </summary>
		public static void Resave()
		{
			foreach(string file in Directory.GetFiles(@"data/noun")) {
				XmlDocument doc = new XmlDocument(); doc.Load(file);
				Noun noun = new Noun(doc);
				FileStream writer = new FileStream(@"data/noun/" + noun.getNickname() + ".xml", FileMode.Create);
				PrettyPrintXml(noun.printXml().DocumentElement.OuterXml, writer);
				writer.Close();
			}
			foreach(string file in Directory.GetFiles(@"data/adjective")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				Adjective adjective=new Adjective(doc);
				FileStream writer=new FileStream(@"data/adjective/"+adjective.getNickname()+".xml", FileMode.Create);
				PrettyPrintXml(adjective.printXml().DocumentElement.OuterXml, writer);
				writer.Close();
			}
			foreach(string file in Directory.GetFiles(@"data/nounPhrase")) {
				XmlDocument doc = new XmlDocument(); doc.Load(file);
				NP np = new NP(doc);
				FileStream writer = new FileStream(@"data/nounPhrase/" + np.getNickname() + ".xml", FileMode.Create);
				PrettyPrintXml(np.printXml().DocumentElement.OuterXml, writer);
				writer.Close();
			}
			foreach(string file in Directory.GetFiles(@"data/preposition")) {
				XmlDocument doc = new XmlDocument(); doc.Load(file);
				Preposition preposition = new Preposition(doc);
				FileStream writer = new FileStream(@"data/preposition/" + preposition.getNickname() + ".xml", FileMode.Create);
				PrettyPrintXml(preposition.printXml().DocumentElement.OuterXml, writer);
				writer.Close();
			}
			foreach(string file in Directory.GetFiles(@"data/verb")) {
				XmlDocument doc = new XmlDocument(); doc.Load(file);
				Verb verb = new Verb(doc);
				FileStream writer = new FileStream(@"data/verb/" + verb.getNickname() + ".xml", FileMode.Create);
				PrettyPrintXml(verb.printXml().DocumentElement.OuterXml, writer);
				writer.Close();
			}
		}

		/// <summary>
		/// Bulk-converts BuNaMo entries from minimal format into expanded format.
		/// Outputs each entry into an individual file.
		/// </summary>
		public static void Go()
		{
			bool doFilter=false;
			List<string> filterNicknames=new List<string>();
			//if(doFilter) filterNicknames=FilterFromNeidTrGrams();
			//if(doFilter) filterNicknames=FilterFromFile(filterNicknames);
			//NB: the nicknames returned by these have been lower-cased
			
			PrinterNeid printer=new PrinterNeid();
			foreach(string file in Directory.GetFiles(@"data/noun")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				Noun noun=new Noun(doc);
				if(!doFilter || filterNicknames.Contains(noun.getNickname().ToLower())) {
					FileStream writer=new FileStream(@"../NeidOutput/"+noun.getNickname()+".xml", FileMode.Create);
					PrettyPrintXml(printer.printNounXml(noun), writer);
					writer.Close();
				}
			}
			foreach(string file in Directory.GetFiles(@"data/adjective")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				Adjective adjective=new Adjective(doc);
				if(!doFilter || filterNicknames.Contains(adjective.getNickname().ToLower())) {
					FileStream writer=new FileStream(@"../NeidOutput/"+adjective.getNickname()+".xml", FileMode.Create);
					PrettyPrintXml(printer.printAdjectiveXml(adjective), writer);
					writer.Close();
				}
			}
			foreach(string file in Directory.GetFiles(@"data/nounPhrase")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				NP np=new NP(doc);
				if(!doFilter || filterNicknames.Contains(np.getNickname().ToLower())) {
					FileStream writer=new FileStream(@"../NeidOutput/"+np.getNickname()+".xml", FileMode.Create);
					PrettyPrintXml(printer.printNPXml(np), writer);
					writer.Close();
				}
			}
			foreach(string file in Directory.GetFiles(@"data/preposition")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				Preposition preposition=new Preposition(doc);
				if(!doFilter || filterNicknames.Contains(preposition.getNickname().ToLower())) {
					FileStream writer=new FileStream(@"../NeidOutput/"+preposition.getNickname()+".xml", FileMode.Create);
					PrettyPrintXml(printer.printPrepositionXml(preposition), writer);
					writer.Close();
				}
			}
			foreach(string file in Directory.GetFiles(@"data/verb")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				Verb verb=new Verb(doc);
				if(!doFilter || filterNicknames.Contains(verb.getNickname().ToLower())) {
					FileStream writer=new FileStream(@"../NeidOutput/"+verb.getNickname()+".xml", FileMode.Create);
					PrettyPrintXml(printer.printVerbXml(verb), writer);
					writer.Close();
				}
			}
		}
		private static List<string> FilterFromFile()
		{
			return FilterFromFile(new List<string>());
		}
		private static List<string> FilterFromFile(List<string> nicknames)
		{
			StreamReader reader=new StreamReader(@"filter.txt");
			while(reader.Peek()>-1) {
				string line=reader.ReadLine().Trim().ToLower();
				if(line!="" && !nicknames.Contains(line)) nicknames.Add(line);
			}
			reader.Close();
			return nicknames;
		}
		private static List<string> FilterFromNeidTrGrams()
		{
			List<string> nicknames=new List<string>();
			XmlTextReader xmlReader=new XmlTextReader(@"2D.xml"); xmlReader.Namespaces=false;
			while(xmlReader.Read()) {
				if(xmlReader.NodeType==XmlNodeType.Element && xmlReader.Name=="Entry") {
					XmlDocument entry=new XmlDocument(); entry.Load(xmlReader.ReadSubtree());
					foreach(XmlElement xmlTrGram in entry.SelectNodes("//TRGRAM[text()!='']")) {
						string trGram=xmlTrGram.InnerText.Trim().ToLower();
						if(!nicknames.Contains(trGram)) nicknames.Add(trGram);
					}
				}
			}
			return nicknames;
		}

		/// <summary>
		/// Bulk-converts BuNaMo entries from minimal format into expanded format.
		/// Combines all entries into a single large file.
		/// </summary>
		public static void GoBulk()
		{
			PrinterNeid printer=new PrinterNeid(false);
			StreamWriter writer;

			writer = new StreamWriter(@"../NeidOutputBulk/nouns.xml");
			writer.WriteLine("<?xml version='1.0' encoding='utf-8'?>");
			writer.WriteLine("<?xml-stylesheet type='text/xsl' href='!lemmas.xsl'?>");
			writer.WriteLine("<lemmas>");
			foreach(string file in Directory.GetFiles(@"data/noun")) {
				XmlDocument doc = new XmlDocument(); doc.Load(file);
				Noun noun = new Noun(doc);
				writer.WriteLine(printer.printNounXml(noun));
			}
			writer.WriteLine("</lemmas>");
			writer.Close();

			writer = new StreamWriter(@"../NeidOutputBulk/nounPhrases.xml");
			writer.WriteLine("<?xml version='1.0' encoding='utf-8'?>");
			writer.WriteLine("<?xml-stylesheet type='text/xsl' href='!lemmas.xsl'?>");
			writer.WriteLine("<lemmas>");
			foreach(string file in Directory.GetFiles(@"data/nounPhrase")) {
				XmlDocument doc = new XmlDocument(); doc.Load(file);
				NP np = new NP(doc);
				writer.WriteLine(printer.printNPXml(np));
			}
			writer.WriteLine("</lemmas>");
			writer.Close();

			writer = new StreamWriter(@"../NeidOutputBulk/adjectives.xml");
			writer.WriteLine("<?xml version='1.0' encoding='utf-8'?>");
			writer.WriteLine("<?xml-stylesheet type='text/xsl' href='!lemmas.xsl'?>");
			writer.WriteLine("<lemmas>");
			foreach(string file in Directory.GetFiles(@"data/adjective")) {
				XmlDocument doc = new XmlDocument(); doc.Load(file);
				Adjective a = new Adjective(doc);
				writer.WriteLine(printer.printAdjectiveXml(a));
			}
			writer.WriteLine("</lemmas>");
			writer.Close();

			writer = new StreamWriter(@"../NeidOutputBulk/prepositions.xml");
			writer.WriteLine("<?xml version='1.0' encoding='utf-8'?>");
			writer.WriteLine("<?xml-stylesheet type='text/xsl' href='!lemmas.xsl'?>");
			writer.WriteLine("<lemmas>");
			foreach(string file in Directory.GetFiles(@"data/preposition")) {
				XmlDocument doc = new XmlDocument(); doc.Load(file);
				Preposition p = new Preposition(doc);
				writer.WriteLine(printer.printPrepositionXml(p));
			}
			writer.WriteLine("</lemmas>");
			writer.Close();

			writer = new StreamWriter(@"../NeidOutputBulk/verbs.xml");
			writer.WriteLine("<?xml version='1.0' encoding='utf-8'?>");
			writer.WriteLine("<?xml-stylesheet type='text/xsl' href='!lemmas.xsl'?>");
			writer.WriteLine("<lemmas>");
			foreach(string file in Directory.GetFiles(@"data/verb")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				Verb v=new Verb(doc);
				writer.WriteLine(printer.printVerbXml(v));
			}
			writer.WriteLine("</lemmas>");
			writer.Close();
		}

		/// <summary>
		/// Lists all entries in BuNaMo.
		/// </summary>
		public static void ListAll()
		{
			StreamWriter writer=new StreamWriter(@"../listAll.txt");
			foreach(string file in Directory.GetFiles(@"data/noun")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				Noun item=new Noun(doc);
				writer.WriteLine("ainmfhocal\t"+item.getLemma()+"\t"+item.getNickname());
			}
			foreach(string file in Directory.GetFiles(@"data/nounPhrase")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				NP item=new NP(doc);
				writer.WriteLine("frása ainmfhoclach\t"+item.getLemma()+"\t"+item.getNickname());
			}
			foreach(string file in Directory.GetFiles(@"data/adjective")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				Adjective item=new Adjective(doc);
				writer.WriteLine("aidiacht\t"+item.getLemma()+"\t"+item.getNickname());
			}
			foreach(string file in Directory.GetFiles(@"data/verb")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				Verb item=new Verb(doc);
				writer.WriteLine("briathar\t"+item.getLemma()+"\t"+item.getNickname());
			}
			foreach(string file in Directory.GetFiles(@"data/preposition")) {
				XmlDocument doc=new XmlDocument(); doc.Load(file);
				Preposition item=new Preposition(doc);
				writer.WriteLine("réamhfhocal\t"+item.getLemma()+"\t"+item.getNickname());
			}
			writer.Close();
		}

		public static string PrettyPrintXml(string doc, FileStream writer)
		{
			XDocument xdoc=XDocument.Load(new StringReader(doc));
			// xdoc.Save(writer, SaveOptions.None);
			using (XmlTextWriter xmlWriter = new XmlTextWriter(writer, System.Text.Encoding.UTF8)) {
				xmlWriter.QuoteChar = '\'';
				xmlWriter.Formatting = Formatting.Indented;
				xmlWriter.Indentation = 1;
				xmlWriter.IndentChar = '\t';
				xdoc.Save(xmlWriter);
			}
			return xdoc.ToString(SaveOptions.None);
		}
		private static string clean4xml(string text)
		{
			string ret = text;
			ret = ret.Replace("&", "&amp;");
			ret = ret.Replace("\"", "&quot;");
			ret = ret.Replace("'", "&apos;");
			ret = ret.Replace("<", "&lt;");
			ret = ret.Replace(">", "&gt;");
			return ret;
		}
	}
}
