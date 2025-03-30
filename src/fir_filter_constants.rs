

// FIR filter constants for upsampling to 192kHz (for 48kHz, 44.1kHz, and 8kHz audio)
pub const FIR_UPSAMPLING_DEG: usize = 12;
pub const FIR_COEFF_48K: [[f32; 12]; 4] = [
    [0.0017089843750, 0.0109863281250, -0.0196533203125, 0.0332031250000, -0.0594482421875, 0.1373291015625, 0.9721679687500, -0.1022949218750, 0.0476074218750, -0.0266113281250, 0.0148925781250, -0.0083007812500],
    [-0.0291748046875, 0.0292968750000, -0.0517578125000, 0.0891113281250, -0.1665039062500, 0.4650878906250, 0.7797851562500, -0.2003173828125, 0.1015625000000, -0.0582275390625, 0.0330810546875, -0.0189208984375],
    [-0.0189208984375, 0.0330810546875, -0.0582275390625, 0.1015625000000, -0.2003173828125, 0.7797851562500, 0.4650878906250, -0.1665039062500, 0.0891113281250, -0.0517578125000, 0.0292968750000, -0.0291748046875],
    [-0.0083007812500, 0.0148925781250, -0.0266113281250, 0.0476074218750, -0.1022949218750, 0.9721679687500, 0.1373291015625, -0.0594482421875, 0.0332031250000, -0.0196533203125, 0.0109863281250, 0.0017089843750]];

pub const FIR_COEFF_44_1K: [[f32; 12]; 4] = [
    [-0.0016626466562397594, 0.0036120560863895854, -0.010049507330194316, 0.02358647878088478, -0.05128342231472882, 0.13264433874805862, 0.9758015015345598, -0.09976902126734796, 0.04215372808543789, -0.019299945146598347, 0.007907304346938194, -0.0027755804829909707],
    [-0.00440747129623946, 0.01140817310152625, -0.03046753034035835, 0.06923176585422793, -0.1518307788766587, 0.4596387688017137, 0.77882617904749, -0.18890593085438728, 0.08393842352561216, -0.037850473319874275, 0.01483731911369804, -0.005283729140918414],
    [-0.005283729140918414, 0.014837319113698028, -0.03785047331987429, 0.08393842352561216, -0.18890593085438728, 0.77882617904749, 0.4596387688017137, -0.1518307788766587, 0.06923176585422791, -0.030467530340358328, 0.01140817310152625, -0.00440747129623946],
    [-0.0027755804829909707, 0.007907304346938197, -0.019299945146598347, 0.04215372808543789, -0.09976902126734796, 0.9758015015345598, 0.1326443387480586, -0.05128342231472882, 0.023586478780884778, -0.01004950733019431, 0.0036120560863895854, -0.0016626466562397594]];

// this uses 24 filters because it is upsampling by a factor of 24 instead of 4 (to get to 192kHz)
pub const FIR_COEFF_8K: [[f32; 12]; 24] = [
    [-0.00027917778647852627, -0.005565134362934542, 0.0005952669663927269, 0.015324928053950842, -0.0016331829680582479, -0.038702367609000074, 0.0037997112591274438, 0.08499301265038789, -0.0081484128443528, -0.18679660917456847, 0.02004066211482792, 0.6547437356079275],
    [-0.0008397559599526656, -0.0056684416910657, 0.0018554700585045763, 0.015713083479874733, -0.005062159060563669, -0.039390256576247853, 0.011708230830744664, 0.08620947140987678, -0.025105684398561742, -0.19044224396868598, 0.06275853055651615, 0.7065482505481093],
    [-0.001399148409208401, -0.005679258456046403, 0.0031949091299048014, 0.01582096235786515, -0.008664091719988585, -0.03937062234087956, 0.01992492700107506, 0.08589391783288332, -0.0427348039906832, -0.19083371435129715, 0.10874722481253557, 0.7556347735623007],
    [-0.0019521565225921614, -0.005588880515134899, 0.004594272536522938, 0.015622060533841693, -0.012379849588908108, -0.03859411369216922, 0.028313145078266412, 0.08395392163255734, -0.06076153909140061, -0.18771410497290691, 0.1576694073521821, 0.801481280871047],
    [-0.002493239325549485, -0.00538933227791899, 0.006030997569672666, 0.01509412953917965, -0.01614351842243662, -0.03702307307303962, 0.036724260091916344, 0.08032125202476272, -0.07888728726420553, -0.18086199345689255, 0.20914059872185442, 0.8435951966138813],
    [-0.003016423169861639, -0.005073684695832214, 0.00747945656757116, 0.014220022786314637, -0.019883299891510373, -0.034633049872564166, 0.044999804105759815, 0.07495451204876272, -0.09679262164378645, -0.1700960356277405, 0.262733176383879, 0.8815200447096196],
    [-0.0035152300923857647, -0.004636389087535364, 0.008911238890755898, 0.012988491731856743, -0.023522597049138855, -0.031414138464648404, 0.05297390045617615, 0.06784143157895513, -0.11414132286080671, -0.15527908363268694, 0.3179810766476167, 0.9148416503900842],
    [-0.003982631289509954, -0.00407361734791797, 0.01029552973006427, 0.011394909796451948, -0.02698127461729838, -0.0273721040009147, 0.060475973962479085, 0.05900076349413508, -0.13058484692054925, -0.1363217593462802, 0.3743851393210184, 0.9431937958095196],
    [-0.004411031464500212, -0.0033835981059032573, 0.011599584216430129, 0.009441902116940362, -0.030177077768886028, -0.02252926142954607, 0.06733369949501228, 0.048483732576117114, -0.1457671695164917, -0.11318541272965968, 0.4314190235766694, 0.9662632422124959],
    [-0.0047922888893428784, -0.0025669376697746334, 0.012789292690470681, 0.007139860027445856, -0.033027188727369095, -0.016925076494720345, 0.07337614509655499, 0.036374992913123044, -0.15932993904993006, -0.08588440343950732, 0.4885356134530629, 0.9837940406960859],
    [-0.005117774902253866, -0.0016269141609988069, 0.0138298303281594, 0.004507320528374884, -0.03544989638773231, -0.010616461537918277, 0.07843706026921948, 0.022793056800255417, -0.1709178634007184, -0.05448765381615886, 0.5451738225837522, 0.9955910644700933],
    [-0.0053784752648456625, -0.0005697331145055542, 0.01468638167212254, 0.001571192900016164, -0.03736635037613159, -0.003677743719925796, 0.08235825519277612, 0.007890166253200487, -0.180184249404651, -0.019119432272965753, 0.6007657003281074, 1.0015227075211368],
    [-0.005565134362934542, 0.0005952669663927269, 0.015324928053950842, -0.0016331829680582479, -0.038702367609000074, 0.0037997112591274438, 0.08499301265038789, -0.0081484128443528, -0.18679660917456847, 0.02004066211482792, 0.6547437356079275, 1.0015227075211368],
    [-0.0056684416910657, 0.0018554700585045763, 0.015713083479874733, -0.005062159060563669, -0.039390256576247853, 0.011708230830744664, 0.08620947140987678, -0.025105684398561742, -0.19044224396868598, 0.06275853055651615, 0.7065482505481093, 0.9955910644700933],
    [-0.005679258456046403, 0.0031949091299048014, 0.01582096235786515, -0.008664091719988585, -0.03937062234087956, 0.01992492700107506, 0.08589391783288332, -0.0427348039906832, -0.19083371435129715, 0.10874722481253557, 0.7556347735623007, 0.9837940406960859],
    [-0.005588880515134899, 0.004594272536522938, 0.015622060533841693, -0.012379849588908108, -0.03859411369216922, 0.028313145078266412, 0.08395392163255734, -0.06076153909140061, -0.18771410497290691, 0.1576694073521821, 0.801481280871047, 0.9662632422124959],
    [-0.00538933227791899, 0.006030997569672666, 0.01509412953917965, -0.01614351842243662, -0.03702307307303962, 0.036724260091916344, 0.08032125202476272, -0.07888728726420553, -0.18086199345689255, 0.20914059872185442, 0.8435951966138813, 0.9431937958095196],
    [-0.005073684695832214, 0.00747945656757116, 0.014220022786314637, -0.019883299891510373, -0.034633049872564166, 0.044999804105759815, 0.07495451204876272, -0.09679262164378645, -0.1700960356277405, 0.262733176383879, 0.8815200447096196, 0.9148416503900842],
    [-0.004636389087535364, 0.008911238890755898, 0.012988491731856743, -0.023522597049138855, -0.031414138464648404, 0.05297390045617615, 0.06784143157895513, -0.11414132286080671, -0.15527908363268694, 0.3179810766476167, 0.9148416503900842, 0.8815200447096196],
    [-0.00407361734791797, 0.01029552973006427, 0.011394909796451948, -0.02698127461729838, -0.0273721040009147, 0.060475973962479085, 0.05900076349413508, -0.13058484692054925, -0.1363217593462802, 0.3743851393210184, 0.9431937958095196, 0.8435951966138813],
    [-0.0033835981059032573, 0.011599584216430129, 0.009441902116940362, -0.030177077768886028, -0.02252926142954607, 0.06733369949501228, 0.048483732576117114, -0.1457671695164917, -0.11318541272965968, 0.4314190235766694, 0.9662632422124959, 0.801481280871047],
    [-0.0025669376697746334, 0.012789292690470681, 0.007139860027445856, -0.033027188727369095, -0.016925076494720345, 0.07337614509655499, 0.036374992913123044, -0.15932993904993006, -0.08588440343950732, 0.4885356134530629, 0.9837940406960859, 0.7556347735623007],
    [-0.0016269141609988069, 0.0138298303281594, 0.004507320528374884, -0.03544989638773231, -0.010616461537918277, 0.07843706026921948, 0.022793056800255417, -0.1709178634007184, -0.05448765381615886, 0.5451738225837522, 0.9955910644700933, 0.7065482505481093],
    [-0.0005697331145055542, 0.01468638167212254, 0.001571192900016164, -0.03736635037613159, -0.003677743719925796, 0.08235825519277612, 0.007890166253200487, -0.180184249404651, -0.019119432272965753, 0.6007657003281074, 1.0015227075211368, 0.6547437356079275]];
