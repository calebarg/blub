package main

// FIXME(calebarg): This needs to run faster + it can't fall over. We need to keep trukin.

import (
	"fmt"
	"path"
	"log"
	"net/http"
	"runtime"
	"time"
	"os"
	"io/fs"
	"strconv"
	"encoding/binary"

	"github.com/PuerkitoBio/gocrawl"
	"github.com/PuerkitoBio/goquery"
)


type Ext struct {
	*gocrawl.DefaultExtender
}

type Blub1Header struct {
	urlLen uint16
	titleLen uint16
	bodyLen uint32
}

var globalExtBlacklistMap = map[string]uint8{".jpg":1, ".png":1, ".xz":1, ".bz2":1, ".asc":1, ".svg":1, ".eps":1, ".phar":1}
var globalRunningID int = 0
var globalSourceDomains = []string{
//"bower.io","cfdocs.org","clojure.org","clojuredocs.org","codecept.io",
//"codeception.com","codeigniter.com",
"coffeescript.org",
"cran.r-project.org","crystal-lang.org","forum.crystal-lang.org","css-tricks.com","dart.dev","dev.mysql.com","developer.apple.com","developer.mozilla.org",
"angular.io","api.drupal.org","api.haxe.org","api.qunitjs.com","babeljs.io","backbonejs.org","bazel.build","bluebirdjs.com",
"developer.wordpress.org","doc.deno.land","doc.rust-lang.org","docs.astro.build","docs.aws.amazon.com","docs.brew.sh","docs.chef.io","docs.cypress.io",
"docs.influxdata.com","docs.julialang.org","docs.microsoft.com","docs.npmjs.com","docs.oracle.com","docs.phalconphp.com","docs.python.org","docs.rs",
"docs.ruby-lang.org","docs.saltproject.io","docs.wagtail.org","doctrine-project.org","docwiki.embarcadero.com","eigen.tuxfamily.org","elixir-lang.org","elm-lang.org",
"en.cppreference.com","enzymejs.github.io","erights.org","erlang.org","esbuild.github.io","eslint.org","expressjs.com","fastapi.tiangolo.com",
"flow.org","fortran90.org","fsharp.org","getbootstrap.com","getcomposer.org","git-scm.com","gnu.org","gnucobol.sourceforge.io",
"go.dev","golang.org","graphite.readthedocs.io","groovy-lang.org","gruntjs.com","handlebarsjs.com","haskell.org","hex.pm",
"hexdocs.pm","httpd.apache.org","i3wm.org","jasmine.github.io","javascript.info","jekyllrb.com","jsdoc.app","julialang.org",
"knockoutjs.com","kotlinlang.org","laravel.com","latexref.xyz","learn.microsoft.com","lesscss.org","love2d.org","lua.org",
"man7.org","mariadb.com","mochajs.org","modernizr.com","momentjs.com","mongoosejs.com","next.router.vuejs.org","next.vuex.vuejs.org",
"nginx.org","nim-lang.org","nixos.org","nodejs.org","npmjs.com","ocaml.org","odin-lang.org","openjdk.java.net",
"opentsdb.net","perldoc.perl.org","php.net","playwright.dev","pointclouds.org","postgresql.org","prettier.io","pugjs.org",
"pydata.org","pytorch.org","qt.io","r-project.org","react-bootstrap.github.io","reactivex.io","reactjs.org",
"reactnative.dev","reactrouterdotcom.fly.dev","readthedocs.io","readthedocs.org","redis.io","redux.js.org","requirejs.org","rethinkdb.com",
"ruby-doc.org","ruby-lang.org","rust-lang.org","rxjs.dev","sass-lang.com","scala-lang.org","scikit-image.org","scikit-learn.org",
"spring.io","sqlite.org","stdlib.ponylang.io","superuser.com","svelte.dev","swift.org","tailwindcss.com","twig.symfony.com",
"typescriptlang.org","underscorejs.org","vitejs.dev","vitest.dev","vuejs.org","vueuse.org","webpack.js.org","wiki.archlinux.org",
"www.chaijs.com","www.electronjs.org","www.gnu.org","www.hammerspoon.org","www.khronos.org","www.lua.org","www.php.net/manual/en/","www.pygame.org",
"www.rubydoc.info","www.statsmodels.org","www.tcl.tk","www.terraform.io","www.vagrantup.com","www.yiiframework.com","yarnpkg.com",}

func (e *Ext) Visit(ctx *gocrawl.URLContext, res *http.Response, doc *goquery.Document) (interface{}, bool) {
  hostName := ctx.NormalizedURL().Hostname()
	blubURL := ctx.NormalizedURL().String()
	blubTitle := doc.Find("title").Text()
	blubBody := doc.Find("body").Text()

	log.Printf("%s\n", blubURL)

	blubFileName := strconv.Itoa(globalRunningID) + ".blub1"
	blubFile, err := os.Create("blub1-data/" + hostName + "/" + blubFileName)
	if err != nil {
		log.Fatal(err)
	}

	blub1Header := Blub1Header{uint16(len(blubURL)), uint16(len(blubTitle)), uint32(len(blubBody))}

	err = binary.Write(blubFile, binary.LittleEndian, blub1Header)
	if err != nil {
		log.Fatal(err)
	}
	err = binary.Write(blubFile, binary.LittleEndian, []byte(blubURL))
	if err != nil {
		log.Fatal(err)
	}
	err = binary.Write(blubFile, binary.LittleEndian, []byte(blubTitle))
	if err != nil {
		log.Fatal(err)
	}
	err = binary.Write(blubFile, binary.LittleEndian, []byte(blubBody))
	if err != nil {
		log.Fatal(err)
	}
	globalRunningID++

	if false {
		var stats runtime.MemStats
		runtime.ReadMemStats(&stats)
		fmt.Printf("HeapAlloc=%02fMB; Sys=%02fMB\n", float64(stats.HeapAlloc)/1024.0/1024.0, float64(stats.Sys)/1024.0/1024.0)
	}

	return nil, true
}

func (e *Ext) Filter(ctx *gocrawl.URLContext, isVisited bool) bool {
	if isVisited {
		return false
	}
	ext := path.Ext(ctx.NormalizedURL().String())
	_, ok := globalExtBlacklistMap[ext]
	if ok {
		log.Printf("Discarding %s\n", ctx.NormalizedURL())
		return false
	}
	return true
}

func main() {
	ext := &Ext{&gocrawl.DefaultExtender{}}
	opts := gocrawl.NewOptions(ext)
	opts.CrawlDelay = 500 * time.Millisecond
	opts.LogFlags = gocrawl.LogError
	opts.SameHostOnly = true
	opts.MaxVisits = 10000
	opts.UserAgent = "BLUB_CRAWLER"

	c := gocrawl.NewCrawlerWithOptions(opts)

	log.Print("BLUB crawler start!")
	for domainIdx := 0; domainIdx < len(globalSourceDomains); domainIdx++ {
		if (domainIdx > 1) {
			break
		}
		domainName := globalSourceDomains[domainIdx]
		log.Printf("Crawling %s\n", domainName)
		err := os.MkdirAll("blub1-data/" + domainName, fs.ModeDir|fs.ModePerm)
		if err != nil {
			log.Fatal(err)
		}
		if err := c.Run("https://" + domainName); err != nil {
			log.Print(err)
		}
	}
}
