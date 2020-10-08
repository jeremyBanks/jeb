import {
  Addon,
  Bundle,
  Game,
  List,
  Subscription,
  getset,
  records,
} from "./data.js";
import {
  digits,
  loadedImage,
  microImageToURL,
  cleanName,
  slugify,
} from "./index.js";
import { jsonObjects } from "./jsons.js";
import {
  fetchStadia,
  checkStatus,
  canFetchStadiaStore,
  canFetchDevApi,
  fetchDevApi,
} from "./net.js";
/** @typedef {import("./data.js").Sku} Sku */
/** @typedef {import("./data.js").Record} Record */

import { sleep, withTimeout } from "./async.js";

const loadSkuData = async (/** @type {Array<unknown>} */ skuData) => {
  const type = {
    1: "game",
    2: "addon",
    3: "bundle",
    5: "subscription",
    6: "addon-subscription",
    10: "preorder",
  }[skuData[6]];
  const skuId = skuData[0];
  const appId = skuData[4];
  const name = skuData[1];

  const untitled = skuData[5];

  const publisherOrganizationId = skuData[15];
  const developerOrganizationIds = skuData[16];

  const coverUrl = skuData[2]?.[1]?.[0]?.[0]?.[1]?.split(/=/)[0];
  const coverMicroData = await microImageFromURL(coverUrl);
  const coverHash = await hashFromURL(coverUrl);

  const releaseDateA = 1000 * skuData[10]?.[0] || undefined;
  const releaseDateB = 1000 * skuData[26]?.[0] || undefined;

  let countries, languages, description;
  if (type === "game") {
    countries = [...skuData[25]].sort();
    languages = [...skuData[24]].sort();
    description = skuData[9];
  }

  const childSkuIds = skuData[14]?.[0]?.map(x => x[0]);
  const childData = skuData[14]?.[0]?.map(x => x[2]);
  if (childData?.filter(Boolean).length) {
    // if this is a shallow view it these will be null
    await Promise.all(childData.map(loadSkuData));
  }

  const props = {
    type,
    skuId,
    appId,
    name,
    coverUrl,
    coverHash,
    countries,
    languages,
    description,
    coverMicroData,
    releaseDateA,
    releaseDateB,
    untitled,
    publisherOrganizationId,
    developerOrganizationIds,
  };

  if (childSkuIds) {
    props.childSkuIds = childSkuIds;
  }

  return getset(props);
};

const keygen = record => {
  if (record.type === "list") {
    return "zzzl" + record._key.padStart(28, "-");
  }

  if (record.type === "organization") {
    return "zzzo" + record.organizationId.slice(0, 28);
  }

  if (record.type === "user") {
    return "zzzu" + record._key.padStart(28, "-");
  }

  let appId = record.appId.replace(/^([a-f0-9]{32})([a-z0-9]+)$/, "$1-$2");
  let skuId = record.skuId.replace(/^([a-f0-9]{32})([a-z0-9]+)$/, "$1-$2");
  let typeTag = "";

  if (record.type === "subscription") {
    appId = "0000";
    typeTag = "SB";
  } else if (record.type === "game") {
    typeTag = "GG";
  } else if (record.type === "addon-subscription") {
    typeTag = "GS";
  } else if (record.type === "addon") {
    typeTag = "GX";
  } else if (record.type === "bundle") {
    typeTag = "PA";
  } else if (record.type === "preorder") {
    typeTag = "PR";
  }

  let idLen = 6;
  let typeLen = 2;
  let nameLen = 32 - typeLen - idLen - idLen;

  let nameTag = (record.slug || slugify(record.name)).replace(/-/g, "");
  if (nameTag.length < nameLen) {
    nameTag += slugify(record.untitled).replace(/-/g, "");
  } else if (nameTag.length > nameLen) {
    // remove last instance of most-frequent letter
    while (nameTag.length > nameLen) {
      const frequencies = {
        0: 0.2,
        e: 0.1249,
        t: 0.0928,
        a: 0.0804,
        o: 0.0764,
        i: 0.0757,
        n: 0.0723,
        s: 0.0651,
        r: 0.0628,
        h: 0.0505,
        l: 0.0407,
        d: 0.0382,
        c: 0.0334,
        u: 0.0273,
        m: 0.0251,
        f: 0.024,
        p: 0.0214,
        g: 0.0187,
        w: 0.0168,
        y: 0.0166,
        b: 0.0148,
        v: 0.0105,
        k: 0.0054,
        x: 0.0023,
        j: 0.0016,
        q: 0.0012,
        z: 0.0009,
      };
      let mostFrequent = "";
      for (const character of nameTag) {
        const frequency = (frequencies[character] += 1);
        if (!mostFrequent || frequency > frequencies[mostFrequent]) {
          mostFrequent = character;
        }
      }

      const index = nameTag.lastIndexOf(mostFrequent);
      nameTag = nameTag.slice(0, index) + nameTag.slice(index + 1);
    }
  }

  const result = [
    appId.padEnd(idLen, 0).slice(0, idLen),
    typeTag.padEnd(typeLen, "?").slice(0, typeLen),
    skuId.padEnd(idLen, 0).slice(0, idLen),
    (nameTag + skuId.slice(idLen)).slice(0, nameLen),
  ].join("");

  if (result.length !== 32) {
    throw new TypeError("wrong id length");
  }
  return result;
};

const spider = async (/** @type {Record} */ record) => {
  const also = {};

  if (record.type === "list") {
    const page = await fetchStadiaPage(`store/list/${record.listId}`);

    if (page.list) {
      for (const sku of page.list) {
        await loadSkuData(sku[9]);
      }
      also.childSkuIds = page.list.map(sku => sku[9][0]);
      also.name = page.heading;
    } else {
      also.childSkuIds = [];
      also.name = page.title.replace(/ - Store - Stadia$/, "");
    }
  } else if (record.type === "user") {
    const page = await fetchStadiaPage(
      `profile/${record.userId}/gameactivities/all`,
    );
    also.playedAppIds = page.playedAppIds.sort();
    also.name = page.user?.[0][0];
    also.number = page.user?.[0][1];
  } else {
    const appId = record.appId || "-";
    const page = await fetchStadiaPage(
      `store/details/${appId}/sku/${record.skuId}`,
    );

    for (const userId of [
      "15010994634532607947",
      "5210057215865025605",
      "7127472037285938012",
      "6237980933414544774",
      "5157635786155057322",
      "5478196876050978967",
      "17731823090785706644",
      "956082794034380385",
      "2015622226121447413",
      "3451791263009607136",
      "534117789100943223",
      "7617790731672886386",
      "5677298969170939332",
      "18009766150536344606",
      "7542096296558587245",
      "689022818271260707",
      "8895703800049233061",
      "17914759789218351281",
      "10018078174943246496",
      "2367383951632393660",
      "8426305397417176748",
      "15260958895835232324",
      "5845011126189582896",
      "15726176272186274158",
      "2014824751648966861",
      "8798383562948483465",
      "7530744285044139557",
      "3818675562075253351",
      "14556924203116610706",
      "16383666641826626246",
      "9538450747714328549",
      "2418188293528093282",
      "5244257281781294963",
      "4114891235384452888",
      "14063326837602612132",
      "803695582367787773",
      "3717994930415920318",
      "9494767075794407876",
      "13717963982377714352",
      "9755354289136709121",
      "879966252234755386",
      "13180849156905876814",
      "7152184736191437026",
      "16611223059862894589",
      "4752098434245298551",
      "5322526905621482226",
      "16920644501578941947",
      "10737520687001829982",
      "16936071534840286009",
      "10119960025946082420",
      "6020863016856436003",
      "2300326898069670223",
      "2619151886572883299",
      "9751430884505163177",
      "18071520400558863008",
      "9113938318949908475",
      "678234406837358983",
      "10754065414324489387",
      "13560220119243150285",
      "17151811203501138830",
      "16635819400168454013",
      "1506015377240366405",
      "18035011556897413468",
      "15315101661505079923",
      "10516342084982777770",
      "4675693367918482410",
      "11931625220400923764",
      "7232419461599664252",
      "9800959408138563826",
      "17552599637848116843",
      "5269088147731986967",
      "1205659528905464572",
      "9023298711363877165",
      "9045593515671378580",
      "16993592771449945591",
      "18359358013222815700",
      "12868146127282324183",
      "13041301274593074982",
      "12610860144979419649",
      "10394577944060863484",
      "14186540238446405913",
      "9016152485229834384",
      "16250781198079395617",
      "13051413772239718427",
      "12141429405693103263",
      "14962700058305101272",
      "9473494818404824125",
      "15992851066294579050",
      "8002431129327450551",
      "7266809003513794352",
      "10177209746089798339",
      "6565346577648093337",
      "1382769713723462939",
      "5873880579270125275",
      "14065052120703633959",
      "2933056940925174408",
      "13314172179757300197",
      "15220733993834473011",
      "12002541963167981793",
      "8048147804215803409",
      "13162550053427529178",
      "2285869566250845110",
      "6236401585125503788",
      "3639730416707727324",
      "13188666408279582206",
      "14096425805807463963",
      "7741089980106411796",
      "44197913558243049",
      "8994937527171073251",
      "10691914222623922114",
      "6387851772671651764",
      "10368989018015580686",
      "7774233538978456849",
      "18191371669598544435",
      "14209423634816457054",
      "2034171637137922456",
      "16902650640432198673",
      "10269254702947687170",
      "16900463116101618359",
      "7196251850914684537",
      "7368994854275987915",
      "7193704168966481219",
      "1703694435016041584",
      "15246784222389626115",
      "13887169259023113979",
      "18139911651253659170",
      "16918545501220947326",
      "757195927764919973",
      "16981259233599418493",
      "3540744592133837974",
      "3074938275569511093",
      "9577444570308464205",
      "3390778050183221224",
      "4253575371920491265",
      "12818174770954654978",
      "16938850672921446804",
      "2678091262326901689",
      "3577894933482649565",
      "16533636656499534317",
      "13851257160529856379",
      "8446386639647628425",
      "7826938077155914270",
      "10030305434603052063",
      "2703520747732146354",
      "5228555431089542960",
      "14392959484228382614",
      "12769762476889393588",
      "4818596636658650423",
      "2871551609626098730",
      "15487299092930718804",
      "7015351721845597218",
      "7126620091754108394",
      "4289518833814721990",
      "4135990210910558308",
      "17023395207714462399",
      "10620416613145881952",
      "1416726824679864800",
      "8853427736310761961",
      "17898880252209235814",
      "8947428180133472267",
      "15010736012600418243",
      "10197485433467279271",
      "1001019704126617365",
      "5745218743461683896",
      "12911839527890423488",
      "13235086755585352569",
      "7748801504126264574",
      "2375793679090750614",
      "15085995584936120212",
      "10963782258356311860",
      "16997595013102527378",
      "7799774572869959224",
      "17632204740785796504",
      "14964802343206759768",
      "16724279401794803522",
      "5837533463983903655",
      "7670375255615906337",
      "4569894695692829679",
      "8664898095290495069",
      "2430746805345475141",
      "1367665640936705130",
      "17632718471186095996",
      "9179143971614497997",
      "2965386521132068545",
      "11241310878145262862",
      "12165576597817349945",
      "7136871491265100006",
      "16396349903981855613",
      "4332442710959854358",
      "1155209660499049669",
      "8207395274397948229",
      "12610550128609689484",
      "2196725011214216825",
      "2977757047165648414",
      "5640544787339604316",
      "9621442136181722654",
      "6241032412305074110",
      "14354787731244753805",
      "7917781742148417899",
      "876192961389596800",
      "13588340882187801336",
      "13140897392651113307",
      "9129445780857559228",
      "13024692158942887296",
      "13668160348272501392",
      "10913677428307408607",
      "9017190419732337347",
      "12809794835651030644",
      "10219822054199365238",
      "1705177642571113420",
      "3845679724155335100",
      "2348662164853285506",
      "13590756441792493776",
      "11690495702777141833",
      "15838461795285203366",
      "1059087052449068676",
      "4458843222298064825",
      "5929032185984208140",
      "523351366059997291",
      "14449574565689873697",
      "9420915141626886835",
      "8231731660218828623",
      "7446208676507265469",
      "5470415440328140724",
      "13057851790739837927",
      "3780661100379605624",
      "1128114917902330071",
      "10273578684398640579",
      "4941963827108645174",
      "12074222993225556283",
      "17154103637806739067",
      "17261980139622037271",
      "11520860907289138861",
      "8715041406645460043",
      "11661297139044279458",
      "8227526994714948446",
      "12994177847429215669",
      "13226435626267671390",
      "1644269501055907282",
      "6377552451777418513",
      "7914242569483008180",
      "1510584562797673835",
      "2981267249581667359",
      "10874155124748530621",
      "5061721990972687360",
      "408559106987538953",
      "6872916923336189448",
      "16484140501357096712",
      "11530028376594832923",
      "12973650184232178424",
      "14090168694025611371",
      "9760772430065906644",
      "2846270809206086320",
      "9169640592810535590",
      "3902384600161307020",
      "14159026780268436927",
      "17666245120610122751",
      "18205250460954481416",
      "10460555075754877840",
      "1924006917530540475",
      "4451475236863599746",
      "1409490496126016331",
      "5500648109846997102",
      "8639051026034178631",
      "12339143863034007009",
      "14465630533673291136",
      "5793334138915161809",
      "8773206467152035497",
      "9144045064192298412",
      "15385480116957511673",
      "17712702501724622129",
      "1063879555844732061",
      "7427551821729454994",
      "2754488850330494720",
      "1601855284510499959",
      "3278120595382939572",
      "14023978675034297027",
      "712039636343876447",
      "6280255736155931123",
      "2451701101277230208",
      "5650328588020277266",
      "742269452978194246",
      "14071391000361956733",
      "1503283120475953249",
      "4219092801866010388",
      "6713952219858161352",
      "3554248871912318616",
      "12199527581659259714",
      "16982083649525320941",
      "2218616119892179993",
      "1250659085418216533",
      "5679210745268091822",
      "15874343148845182232",
      "1393961112127541268",
      "5315832739311004143",
      "976867315257417760",
      "5983484977401616223",
      "5570082903862061646",
      "2120671477126250841",
      "16068953871908775753",
      "10300630881989178988",
      "14009379736560733706",
      "7490078680400014766",
      "16198040833007882619",
      "5595110285334832739",
      "16592872878100305892",
      "6056079670761737249",
      "3265593822481686296",
      "8701068685184883544",
      "582225974977521149",
      "6358707458280597992",
      "191616488992247066",
      "18049803131943310230",
      "11302424249211435398",
      "14408875234214174980",
      "3992866969498550706",
      "16628303516936418804",
      "4022410443812959468",
      "1085762710972950906",
      "4461862536199835034",
      "333505726213544319",
      "14360247565077533009",
      "13747486024590636110",
      "2004126601886559441",
      "10577308421307276273",
      "2300076809035974004",
      "6534793296471275205",
      "13409074072243923713",
      "10821996493450713030",
      "16468603283263119392",
      "6124538546538368812",
      "6940615283644370800",
      "16077040128822346074",
      "6473752946605115142",
      "79892041887079382",
      "3178985571134471442",
      "14883893893511019400",
      "12228852059126414610",
      "166361702571018664",
      "9226532719219852064",
      "6523761630865578555",
      "5823985731925680934",
      "7196283175797010807",
      "12433287953891139317",
      "10557443705670180345",
      "14077257780980091352",
      "17138559766455083985",
      "6738593952754951018",
      "3501725703279423676",
      "4494128663969078070",
      "1037847632787352515",
      "9086456813605343525",
      "16861082082698305052",
      "17088485202864272086",
      "10683728468081092459",
      "5144785982937267899",
      "4663312335031233589",
      "1068057532476288823",
      "17797085350648398473",
      "15915297138693016069",
      "14937383885734667311",
      "8222700421413599415",
      "2693482689586955312",
      "1685857834070174822",
      "4984110636870407118",
      "8659210114068582184",
      "987911440460259131",
      "8953672885412680619",
      "6829710370739863606",
      "13260301474319675102",
      "15038059824524706660",
      "18122064360048698516",
      "13985473264129093445",
      "10775436197650657517",
      "10952975179945028979",
      "10723259664692243923",
      "12386058221100096962",
      "607433462798044623",
      "2452837000545364327",
      "16288041368175040389",
      "17442211575525960433",
      "168286212330069133",
      "17062334576206518198",
      "4857055573382954254",
      "13045987358465993067",
      "13810573943327641147",
      "10412539040395513028",
      "3813171934975076817",
      "3169316724019330369",
      "3413084449073121748",
      "691859615736962736",
      "16271216632873383642",
      "8059889775104255671",
      "11351115672760772107",
      "16754052409179674169",
      "6892205731007696190",
      "1081586047498839031",
      "10415816353486888818",
      "14736367409521557403",
      "12958869881372163199",
      "14818786020555567982",
      "15701828454036745197",
      "9205373918089128755",
      "14998845255978084250",
      "3977593763480184064",
      "7558693484544943235",
      "14543092907105346918",
      "11794771645534304686",
      "10551210966105871283",
      "5568820551194799992",
      "3096875866283963783",
      "1162428908367464029",
      "7641666874362244451",
      "2786031202905461251",
      "86598125954858820",
      "4809028534248221103",
      "18017902877012151756",
      "8332293769868657588",
      "8743728498803553863",
      "13557984063261473886",
      "15589591416590325227",
      "15684549473745682829",
      "11634657608543545778",
      "11481877782164821727",
    ]) {
      getset({
        type: "user",
        userId,
      });
    }

    const organizations = [page.sku[22][0], ...page.sku[22][1]];
    for (const organization of organizations) {
      getset({
        type: "organization",
        organizationId: organization[0],
        name: organization[2][0],
      });
    }

    await loadSkuData(page.sku[16]);
    if (page.gameAddons) {
      for (const sku of page.gameAddons) {
        await loadSkuData(sku[9]);
      }
    }
    if (page.gameBundles) {
      for (const sku of page.gameBundles) {
        await loadSkuData(sku[9]);
      }
    }
    if (page.gameSubscriptions) {
      for (const sku of page.gameSubscriptions) {
        await loadSkuData(sku[9]);
      }
    }
  }

  getset({
    ...record,
    ...also,
    lastSpidered: Date.now(),
  });
};

const inclusive = (a, b) => {
  const r = new Array();
  for (let i = a; i >= a && i <= b; i++) {
    r.push(i);
  }
  return r;
};

/** @returns {Promise<unknown>} */
export const spiderThread = async () => {
  try {
    await withTimeout(16, canFetchStadiaStore);
  } catch (error) {
    console.debug(
      "Failed to connect to Stadia store, abandoning spider.",
      error,
    );
    return;
  }

  getset({
    type: "list",
    listId: 3,
  });

  getset({
    type: "subscription",
    skuId: "59c8314ac82a456ba61d08988b15b550",
  });

  try {
    await withTimeout(16, canFetchDevApi);

    const skus = await (await fetchDevApi("skus.json")).json();
    const meta = await (await fetchDevApi("skus-meta.json")).json();
    for (const key of Object.keys(skus)) {
      Object.assign(skus[key], meta[key]);
    }
    Object.values(skus).forEach(getset);
    console.info(`${Object.keys(records).length} records loaded.`, records);
  } catch (error) {
    console.debug(
      "Failed to connect to local dev server, skipping load.",
      error,
    );
  }

  for (;;) {
    const now = Date.now();
    const allRecords = Object.values(records).sort((a, b) => {
      if (a.age(now) < b.age(now)) {
        return +1;
      } else if (b.age(now) < a.age(now)) {
        return -1;
      } else if (a.lastModified < b.lastModified) {
        return -1;
      } else if (b.lastModified < a.lastModified) {
        return +1;
      } else {
        return 0;
      }
    });

    const ageLimit = 24 * 60 * 60 * 1000;

    const staleRecords = allRecords.filter(
      record => record.age(now) > ageLimit,
    );

    const record = allRecords[0];

    const icon =
      allRecords.length > 8 ? (staleRecords.length > 0 ? "⚠️" : "✅") : "❌";

    document.querySelector(
      "#dev-tools .record-count",
    ).textContent = `${icon} ${staleRecords.length} stale of ${allRecords.length} total`;

    if (staleRecords.length === 0) {
      console.info(
        `Everything has been spidered recently (at most ${
          record.age(now) / 1000 / 60 / 60
        } hours ago).`,
      );
      await sleep(128.0);
      continue;
    }

    await updateDocument();

    if (await canFetchDevApi) {
      const skus = {};
      const meta = {};
      for (const key of Object.keys(records).sort()) {
        const item = records[key];
        const newKey = keygen(item);

        if (skus.hasOwnProperty(newKey)) {
          throw new Error(`duplicate key ${newKey}`);
        }

        skus[newKey] = {
          appId: item.appId,
          languages: item.languages,
          description: item.description,
          countries: item.countries,
          childSkuIds: item.childSkuIds,
          coverHash: item.coverHash,
          coverMicroData: item.coverMicroData,
          coverUrl: item.coverUrl,
          developerOrganizationIds: item.developerOrganizationIds,
          isPro: item.isPro,
          listId: item.listId,
          name: item.name,
          number: item.number,
          organizationId: item.organizationId,
          playedAppIds: item.playedAppIds,
          publisherOrganizationId: item.publisherOrganizationId,
          releaseDateA: item.releaseDateA,
          releaseDateB: item.releaseDateB,
          skuId: item.skuId,
          slug: item.slug,
          type: item.type,
          untitled: item.untitled,
          userId: item.userId,
          wasPro: item.wasPro,
        };

        meta[newKey] = {
          firstSeen: item.firstSeen,
          lastModified: item.lastModified,
          lastSeen: item.lastSeen,
          lastSpidered: item.lastSpidered,
        };
      }
      fetchDevApi("skus.json", {
        method: "PUT",
        body: JSON.stringify(skus, null, 2),
      });
      fetchDevApi("skus-meta.json", {
        method: "PUT",
        body: JSON.stringify(meta, null, 2),
      });
      downloadDocument();
    }

    await spider(record);
    console.info("🕷️ spidered", record);
    console.debug(`${Object.keys(records).length} records.`, records);
    await sleep(6.0);
  }
};

const fetchStadiaPage = async url => {
  const opaque = await fetchStadiaOpaque(url);
  const data = Object.create(opaque);

  data.heading = opaque.HZ5mJ;
  data.title = opaque.title;
  data.self = opaque.D0Amudob?.[5];
  data.user = opaque.D0Amudoboos?.[5];
  data.list = opaque.WwD3rbnob?.[2];
  data.gameStats = opaque.e7h9qdoss?.[0]?.[8];
  data.playedAppIds = opaque.Q6jt8cooos?.[0];
  data.sku = opaque.FWhQVssb;
  data.storefront = opaque.xjyeoc?.[3].flatMap(x => x?.[1]);
  data.gameAddons = opaque.ZAm7Wesooooooob?.[0];
  data.gameBundles = opaque.SYcsTdsb?.[1];
  data.gameSubscriptions = opaque.SYcsTdsb?.[2];

  for (const key of Object.keys(data)) {
    if (data[key] === undefined) {
      delete data[key];
    }
  }

  console.debug("Got Stadia page", data);

  return data;
};

const padOpaqueKeys = (/** @type {unknown} */ object) => {
  if (
    object &&
    typeof object === "object" &&
    !(object instanceof Array) &&
    Object.keys(object).length >= 4 &&
    Object.keys(object).every(key => /^[a-zA-Z0-9]{1,6}$/.test(key))
  ) {
    return Object.fromEntries(
      Object.entries(object).map(([key, value]) => [key.padEnd(6, "s"), value]),
    );
  } else {
    return object;
  }
};

const fetchStadiaOpaque = async url => {
  const response = await fetchStadia(url);
  console.debug("Got Stadia response", response);
  checkStatus(response);

  const body = await response.text();
  const doc = new DOMParser().parseFromString(body, "text/html");

  const scripts = [...doc.querySelectorAll("script")];

  const jsons = scripts.flatMap(script => jsonObjects(script.textContent));

  const data = Object.create(jsons);

  for (const el of doc.querySelectorAll("[role][class]")) {
    data[el.className] = el.textContent;
  }

  data.title = doc.querySelector("title")?.textContent;

  Object.assign(
    data,
    padOpaqueKeys(
      jsons.find(
        x =>
          Object.keys(x).length >= 8 &&
          Object.keys(x).every(key => /^[a-zA-Z0-9]{1,6}$/.test(key)),
      ),
    ),
  );

  const preloadQueries = jsons.find(x => x?.["ds:0"]?.["id"]);

  if (preloadQueries) {
    for (const [key, { id, request }] of Object.entries(preloadQueries)) {
      const preloadResponse = jsons.find(x => x.key === key);
      const response = preloadResponse?.data;
      const name =
        request.length > 0
          ? id + request.map(x => (typeof x).slice(0, 1)).join("")
          : id;
      data[name] = response;
    }
  }

  Object.assign(
    data,
    Object.fromEntries(
      jsons
        .find(
          ({ values }) =>
            values instanceof Array &&
            values.length >= 4 &&
            values.includes("stadia.google.com") &&
            values.includes("https://stadia.google.com/"),
        )
        ?.values.map((value, i) => [
          ((i + 7577) / 7919)
            .toString(36)
            .replace(/[^A-Za-z]+/, "")
            .slice(0, 6)
            .padEnd(6, "s"),
          value,
        ]) || [],
    ),
  );

  if (data.nQyAEs) {
    Object.assign(data, padOpaqueKeys(data.nQyAEs));
    delete data.nQyAEs;
  }

  return data;
};

/**
 * Returns an base-64 encoded 8x8 thumbnail the image at a given URL.
 * @returns {Promise<String>}
 */
const microImageFromURL = async (/** @type string */ url) => {
  const image = await loadedImage(url);
  const canvas = document.createElement("canvas");
  canvas.width = 8;
  canvas.height = 8;
  const g2d = canvas.getContext("2d");
  g2d.drawImage(image, 0, 0, canvas.width, canvas.height);
  const pixels = g2d.getImageData(0, 0, canvas.width, canvas.height);

  const microImage = new Array();
  for (let i = 0; i < 64; i++) {
    const rgb = pixels.data.slice(i * 4, i * 4 + 3);
    const u6 = rgbToU6(rgb);
    microImage.push(digits[u6]);
  }

  return microImage.join("");
};

const hashFromURL = async (/** @type string */ url) => {
  const response = await fetch(url);
  const body = await response.arrayBuffer();
  const hash = await crypto.subtle.digest("SHA-512", body);
  const byteLength = 8;
  const hexHash = Array.from(new Uint8Array(hash))
    .slice(0, byteLength)
    .map(b => b.toString(16).padStart(2, "0"))
    .join("");
  return `f12${byteLength.toString(16).padStart(2, "0")}${hexHash}`;
};

/**
 * Rounds a 24-bit RGB value to the nearest 6-bit RGB value.
 * @returns {number}
 */
const rgbToU6 = (/** @type [number, number, number] */ rgb) => {
  const red = Math.round((0b11 * rgb[0]) / 0xff);
  const green = Math.round((0b11 * rgb[1]) / 0xff);
  const blue = Math.round((0b11 * rgb[2]) / 0xff);
  return (red << 0) + (green << 2) + (blue << 4);
};

const downloadDocument = async () => {
  const docToDownload = document.documentElement.cloneNode(true);

  docToDownload.querySelector("title").textContent = "stadia.run";

  docToDownload.querySelector("base").removeAttribute("target");

  for (const el of docToDownload.querySelectorAll("[hidden]")) {
    el.removeAttribute("hidden");
  }

  for (const input of docToDownload.querySelectorAll("input[value]")) {
    el.removeAttribute("value");
  }

  for (const el of docToDownload.querySelectorAll("[style]")) {
    el.removeAttribute("style");
  }

  for (const el of docToDownload.querySelectorAll('[class=""],main [class]')) {
    el.removeAttribute("class");
  }

  for (const el of docToDownload.querySelectorAll(
    ".dev-server-status,.stadia-proxy-status,.record-count",
  )) {
    el.textContent = "❓";
  }

  const html =
    "<!doctype html>" +
    docToDownload.innerHTML
      .replace(/\s*<\/body>\s*$/, "\n")
      .replace(/^<head>/, "")
      .replace(/<\/head><body>/, "")
      .replace(/(\s)(disabled|autofocus|pre-order|pro)(="")([>\s])/g, "$1$2$4");

  await fetch("//dev-api.stadia.st:57482/index.html", {
    method: "PUT",
    body: html,
  });
};

const updateDocument = async () => {
  const games = [...Object.values(records)]
    // only include games that are to be released within the next week
    .filter(sku => sku.type === "game")
    .map(game => ({
      ...game,
      slug: game.slug,
      name: cleanName(game.name),
      preOrder:
        Math.max(game.releaseDateA, game.releaseDateB) >
        Date.now() + 1000 * 60 * 60 * 24 * 2,
    }))
    .sort((gameA, gameB) => {
      const aFirst = -1;
      const bFirst = +1;

      const aName = gameA.name.toLowerCase();
      const bName = gameB.name.toLowerCase();

      const aReleased = Math.max(gameA.releaseDateA, gameA.releaseDateB);
      const bReleased = Math.max(gameB.releaseDateA, gameB.releaseDateB);

      if (gameA.preOrder && !gameB.preOrder) {
        return bFirst;
      } else if (!gameA.preOrder && gameB.preOrder) {
        return aFirst;
      } else if (gameA.preOrder && gameB.preOrder) {
        if (aReleased > bReleased) {
          return bFirst;
        } else if (aReleased < bReleased) {
          return AFirst;
        }
      } else if (gameA.isPro && !gameB.isPro) {
        return aFirst;
      } else if (!gameA.isPro && gameB.isPro) {
        return bFirst;
      } else if (aReleased > bReleased) {
        return aFirst;
      } else if (aReleased < bReleased) {
        return bFirst;
      } else if (gameA.wasPro && !gameB.wasPro) {
        return aFirst;
      } else if (!gameA.wasPro && gameB.wasPro) {
        return bFirst;
      } else if (aName < bName) {
        return aFirst;
      } else if (aName > bName) {
        return bFirst;
      } else {
        return 0;
      }
    });

  const template = document.querySelector("st-games template");

  const fragment = document.createDocumentFragment();

  const request = await fetch("//dev-api.stadia.st:57482/manifest.json", {
    method: "GET",
  });
  const manifest = await request.json();

  manifest.shortcuts = [];

  for (const game of games) {
    let root = template.content.cloneNode(true).firstElementChild;
    let url = game.coverUrl;

    if (manifest.shortcuts.length < 16) {
      manifest.shortcuts.push({
        name: game.name,
        url: `/${game.slug}`,
        icons: [
          {
            src: url + "=s192-p-rp",
            sizes: "192x192",
          },
        ],
      });
    }

    const fullImg = root.querySelector("img");
    fullImg.src = url + "=w640-h360-rw";
    root.querySelector("st-cover-full").hidden = fullImg.complete;
    root.querySelector("st-cover-micro").hidden = !fullImg.complete;
    loadedImage(url)
      .then(() => {
        root.querySelector("st-cover-full").hidden = false;
        root.querySelector("st-cover-micro").hidden = true;
      })
      .catch(error => console.error(error));

    const link = root.querySelector("a");
    link.href = `https://stadia.google.com/player/${game.appId}`;

    const name = root.querySelector("st-name");
    name.textContent = game.name;

    const slug = root.querySelector("st-slug");
    slug.textContent = "/" + game.slug;

    root.querySelector(
      "st-cover-micro",
    ).style.backgroundImage = `url(${microImageToURL(game.coverMicroData)})`;

    root
      .querySelector("st-cover-micro")
      .setAttribute("data", game.coverMicroData);

    if (game.isPro) {
      const badge = Object.assign(document.createElement("st-badge"), {
        textContent: "PRO",
      });
      badge.setAttribute("pro", "");
      link.appendChild(badge);
    } else if (game.wasPro) {
      const badge = Object.assign(document.createElement("st-badge"), {
        innerHTML: "previously<br />PRO",
      });
      badge.setAttribute("previously-pro", "");
      link.appendChild(badge);
    }

    if (game.preOrder) {
      const badge = Object.assign(document.createElement("st-badge"), {
        textContent: "pre-order",
      });
      badge.setAttribute("pre-order", "");
      link.appendChild(badge);
    }

    fragment.appendChild(document.createTextNode("\n    "));
    fragment.appendChild(root);
  }

  template.remove();
  const gamesEl = document.querySelector("st-games");
  gamesEl.textContent = "";
  gamesEl.appendChild(template);
  gamesEl.appendChild(fragment);

  gamesEl.appendChild(document.createTextNode("\n  "));

  await fetch("//dev-api.stadia.st:57482/manifest.json", {
    method: "PUT",
    body: JSON.stringify(manifest, null, 2),
  });
};
