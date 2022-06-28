const SVG_NS = 'http://www.w3.org/2000/svg';
const SVG_DEFS = [
	'<g id="empty"/>',
	'<g id="x"><path class="glyph x" d="M 0.1 0.1 L 0.9 0.9 M 0.9 0.1 L 0.1 0.9"></g>',
	'<g id="o"><circle class="glyph o" cx="0.5" cy="0.5" r="0.4"/></g>',
].join('\n');

const MARKS = ['empty', 'x', 'o'];

class Board {
	constructor(width, height) {
		const svg = document.createElementNS(SVG_NS, 'svg');
		svg.setAttribute('viewBox', '0 0 ' + width + ' ' + height);
		svg.style.width = width + 'in';
		svg.style.height = height + 'in';
		svg.addEventListener('click', this.onClick.bind(this));

		const defs = document.createElementNS(SVG_NS, 'defs');
		defs.innerHTML = SVG_DEFS;
		svg.appendChild(defs);

		const spots = [];
		for(let y = 0; y < height; y++) {
			const spy = [];
			for(let x = 0; x < width; x++) {
				const spot = document.createElementNS(SVG_NS, 'g');
				spot.setAttribute('class', 'spot');
				spot.setAttribute('transform', 'translate(' + x + ' ' + y + ')');
				svg.appendChild(spot);
				spy.push(spot);

				const sq = document.createElementNS(SVG_NS, 'rect');
				sq.setAttribute('x', 0); sq.setAttribute('y', 0);
				sq.setAttribute('width', 1); sq.setAttribute('height', 1);
				sq.setAttribute('class', 'space');
				spot.appendChild(sq);

				const mk = document.createElementNS(SVG_NS, 'use');
				mk.setAttribute('href', '#empty');
				mk.setAttribute('class', 'mark');
				spot.appendChild(mk);
			}
			spots.push(spy);
		}

		this.svg = svg;
		this.spots = spots;
	}

	spot(x, y) {
		const row = this.spots[y];
		if(row == undefined) return;
		return row[x];
	}

	set(x, y, mark) {
		const spot = this.spot(x, y);
		if(spot == undefined) return;
		spot.querySelector('.mark').setAttribute('href', '#'+mark);
	}

	get(x, y) {
		const spot = this.spot(x, y);
		if(spot == undefined) return;
		return spot.querySelector('.mark').getAttribute('href').substring(1);
	}

	screenToBoard(x, y) {
		const pt = new DOMPointReadOnly(x, y);
		return pt.matrixTransform(this.svg.getScreenCTM().inverse());
	}

	onClick(ev) {
		let {x, y} = this.screenToBoard(ev.clientX, ev.clientY);
		console.log("click @%f,%f -> %f,%f", ev.clientX, ev.clientY, x, y);
		[x, y] = [x, y].map(Math.floor);
		//let idx = MARKS.find(this.get(x, y));
		const mark = this.get(x, y);
		let idx = MARKS.indexOf(mark);
		idx += 1;
		if(idx >= MARKS.length) idx = 0;
		this.set(x, y, MARKS[idx]);
	}
}

let board;

const init = () => {
	board = new Board(3, 3);
	document.body.appendChild(board.svg);
};

document.addEventListener("DOMContentLoaded", init);